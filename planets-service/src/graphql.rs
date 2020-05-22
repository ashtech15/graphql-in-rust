use std::ops::Mul;
use std::str::FromStr;

use async_graphql::*;
use bigdecimal::FromPrimitive;
use num_bigint::*;

use crate::db::{DetailsEntity, NewDetailsEntity, NewPlanetEntity, PlanetEntity};
use crate::db_connection::PgPool;
use crate::model::{BigDecimal, BigInt, Details, DetailsInput, InhabitedPlanetDetails, Planet, PlanetType, UninhabitedPlanetDetails};
use crate::repository;

pub type TestSchema = Schema<Query, Mutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    async fn planets(&self, ctx: &Context<'_>) -> Vec<Planet> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let db_planets = repository::all(&conn).expect("Can't get planets");

        db_planets.iter()
            .map(|(p, d)| { convert(p, d) })
            .collect()
    }

    #[entity]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let db_planets = repository::all(&conn).expect("Can't get planets");

        let found = db_planets.iter()
            .find(|(p, _)| {
                p.id == id.to_string().parse::<i32>().expect("Can't get ID from String")
            });

        found.map(|(p, d)| { convert(p, d) })
    }
}

// todo from/into trait
fn convert(db_planet: &PlanetEntity, db_details: &DetailsEntity) -> Planet {
    let details: Details = if db_details.population.is_some() {
        InhabitedPlanetDetails {
            mean_radius: BigDecimal(db_details.mean_radius.clone()),
            mass: BigInt(db_details.mass.to_bigint().clone().expect("Can't get mass")),
            population: BigDecimal(db_details.population.as_ref().expect("Can't get population").clone()),
        }.into()
    } else {
        UninhabitedPlanetDetails {
            mean_radius: BigDecimal(db_details.mean_radius.clone()),
            mass: BigInt(db_details.mass.to_bigint().clone().expect("Can't get mass")),
        }.into()
    };

    Planet {
        id: db_planet.id.into(),
        name: db_planet.name.clone(),
        planet_type: PlanetType::from_str(db_planet.planet_type.as_str()).expect("Can't convert string to enum"),
        details,
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_planet(&self, ctx: &Context<'_>, name: String, planet_type: PlanetType, details: DetailsInput) -> ID {
        fn get_new_planet_mass(number: f32, ten_power: usize) -> bigdecimal::BigDecimal {
            let some = bigdecimal::BigDecimal::from(number);
            some.mul(num::pow(bigdecimal::BigDecimal::from(10), ten_power))
        }

        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let new_planet = NewPlanetEntity {
            name,
            planet_type: planet_type.to_string(),
        };

        let new_planet_details = NewDetailsEntity {
            mean_radius: details.mean_radius.0,
            mass: get_new_planet_mass(details.mass.number, details.mass.ten_power as usize),
            population: details.population.map(|v| { v.0 }),
            planet_id: 0,
        };

        let create_planet_result = repository::create(new_planet, new_planet_details, &conn);

        create_planet_result.expect("Can't create new planet").id.into()
    }
}