use bevy::{
    core_pipeline::tonemapping::Tonemapping, ecs::world, prelude::*, reflect::TypeRegistry,
};
use bevy_granite_core::{EditorIgnore, entities};
use rand::SeedableRng;

use crate::{
    MetaData,
    reflect_serde::test::data_generation::EntitiesForTest,
    scene::{HumanReduced, HumanVerbose},
};

mod data_generation;

#[test]
fn enums() {
    let mut app = App::new();
    app.insert_resource(Seed(42));
    app.add_systems(Startup, data_generation::spawn_entitys);
    app.init_resource::<EntitiesForTest>();
    app.update();
    let world = app.world();
    let entities = world.resource::<data_generation::EntitiesForTest>();
    let mut out = String::new();
    let components = world.components();
    let reg = world.resource::<AppTypeRegistry>().clone();
    let reg = reg.read();
    let meta = MetaData::default();
    let mut data = HumanVerbose;
    let mut entity_serde =
        super::EntitySerializer::new(&reg, components, &mut out, 0, &meta, &mut data);
    for (entity, _) in entities.entitys.iter() {
        entity_serde.serialize_entity(*entity, &world);
    }
    println!("Serialized:\n\n\n{}", out);
}

#[test]
fn world_saver_verbose() {
    let mut app = App::new();
    app.insert_resource(Seed(42));
    app.init_resource::<EntitiesForTest>();
    app.add_systems(Startup, data_generation::spawn_entitys);
    crate::reflect_serde::register_garnet_serialize_types(&mut app);

    app.update();
    let world = app.world();
    let serialiser = crate::scene::SceneSaver::new::<HumanVerbose>(
        world,
        format!(
            "{}/../../assets/scenes/verbose_scene.garnet",
            std::env::current_dir().unwrap().display()
        ),
    )
    .expect("can create scene saver");
    serialiser.serialize_world().expect("can serialize world");

    let mut app_new = App::new();
    app_new.insert_resource(Seed(42));
    app_new.init_resource::<EntitiesForTest>();
    app_new.add_systems(Startup, data_generation::spawn_entitys);
    crate::reflect_serde::register_garnet_serialize_types(&mut app_new);

    app_new.update();

    let world = app_new.world_mut();
    let mut loader = crate::scene::SceneLoader::new(
        world,
        format!(
            "{}/../../assets/scenes/verbose_scene.garnet",
            std::env::current_dir().unwrap().display()
        ),
    )
    .expect("can create scene loader");
    loader.load_scene().expect("can load scene");

    let meta = loader.crack();
    app_new.update();

    // check entities in world a = world b
}

#[test]
fn world_saver_reduced() {
    let mut app = App::new();
    app.insert_resource(Seed(42));
    app.init_resource::<EntitiesForTest>();
    app.add_systems(Startup, data_generation::spawn_entitys);
    crate::reflect_serde::register_garnet_serialize_types(&mut app);

    app.update();
    let world = app.world();
    let path = format!(
        "{}/../../assets/scenes/reduced_scene.garnet",
        std::env::current_dir().unwrap().display()
    );
    println!("Saving to path: {path}");
    let serialiser =
        crate::scene::SceneSaver::new::<HumanReduced>(world, path).expect("can create scene saver");
    serialiser.serialize_world().expect("can serialize world");

    let mut app_new = App::new();
    app_new.insert_resource(Seed(42));
    app_new.init_resource::<EntitiesForTest>();
    app_new.add_systems(Startup, data_generation::spawn_entitys);
    crate::reflect_serde::register_garnet_serialize_types(&mut app_new);

    app_new.update();

    let world = app_new.world_mut();
    let mut loader = crate::scene::SceneLoader::new(
        world,
        format!(
            "{}/../../assets/scenes/reduced_scene.garnet",
            std::env::current_dir().unwrap().display()
        ),
    )
    .expect("can create scene loader");

    loader.load_scene().expect("can load scene");

    let meta = loader.crack();
    panic!(); // so console has output from test,todo remove when it all works
    app_new.update();

    // check entities in world a = world b
}

#[derive(Resource)]
struct Seed(u64);
