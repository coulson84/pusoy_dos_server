use iron::prelude::*;
use iron::status;
use iron::middleware::Handler;
use iron::mime::Mime;
use router::Router;

use tera::{Tera, Context};
use data_access::round::Round as RoundData;
use config::Config;
use helpers;
use pusoy_dos::game::game::Game;
use pusoy_dos::cards::card::Card;
use serde::{Serialize, Serializer};

pub struct InPlay{
    tera: &'static Tera,
    round_data: RoundData,
    hostname: String
}

impl InPlay {
    
    pub fn new(config: &Config, tera:&'static Tera, round_data: RoundData) -> InPlay {

        let hostname = config.get("hostname").unwrap();

        InPlay{
            tera: tera,
            round_data: round_data,
            hostname: hostname
        }
    }

    pub fn display(&self, user_id:u64, game_id:u64) -> Response {

        let template = "inplay.html";
        let mut data = Context::new();
        let round_result = self.round_data.get(game_id);

        match round_result {
            None => {
                info!("redirecting as no round found for game {}", game_id);
                return helpers::redirect(&self.hostname, "games");  // think about an error page here?
            },
            _ => ()
        }

        let round = round_result.unwrap();
        info!("loading game: {}", game_id);

        let game = Game::load(round).unwrap();
        info!("game loaded");

        let next_player = game.get_next_player().unwrap();

        let current_user_turn = user_id == next_player.get_id(); 
        let current_user = game.get_player(user_id).unwrap();

        let cards:Vec<DCard> = current_user.get_hand().iter().map(|&c|{ DCard(c.clone()) }).collect();

        data.add("your_turn", &current_user_turn);
        data.add("id", &game_id);
        data.add("cards", &cards);
        let content_type = "text/html".parse::<Mime>().unwrap();
        let page = self.tera.render(template, data).unwrap();
        Response::with((content_type, status::Ok, page))
    }
}

impl Handler for InPlay {

    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let ref query = req.extensions.get::<Router>().unwrap().find("id");

        let session_user_id = helpers::get_user_id(req);
        let redirect_to_homepage = helpers::redirect(&self.hostname, "");

        let resp = match session_user_id {
            Some(user_id) => {
                match *query {
                    Some(id) => {
                        self.display(user_id, id.parse::<u64>().unwrap())
                    },
                    _ => redirect_to_homepage
                }
            },
            _ => redirect_to_homepage
        };

        Ok(resp)
    }

}

// todo - move
//
struct DCard(Card);

impl Serialize for DCard {

	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {

        let mut state = try!(serializer.serialize_map(Some(2)));
		try!(serializer.serialize_map_key(&mut state, "suit_display"));
		try!(serializer.serialize_map_value(&mut state, format!("{}", self.0.suit)));
		try!(serializer.serialize_map_key(&mut state, "suit"));
		try!(serializer.serialize_map_value(&mut state, format!("{:?}", self.0.suit)));
		try!(serializer.serialize_map_key(&mut state, "rank"));
		try!(serializer.serialize_map_value(&mut state, format!("{}", self.0.rank)));
        serializer.serialize_map_end(state)
    }
}
