# Use Tiled custom properties

In Tiled we can add "custom properties" on various items such as layers, tiles, objects or maps.

These custom properties can be either:

- a "standard type", like a string, an integer, a float, a color, etc...
- a "custom type", which is basically a structure with sub-properties that can either be a "standard type" or another "custom type"

Using `bevy_ecs_tiled`, you can load these "custom properties" in your game and access them as regular Bevy `Component`, `Bundle` or even `Resource`.
Basically, it means that we can define some game logic directly in the Tiled editor to use it in your game with Bevy.

Using this mechanism, we could for instance:

- associate a "movement cost" to a given tile type
- create an object that represent our player or an ennemy
- add a generic "trigger zone" that could either be a "damaging zone" or a "victory zone"
- ... whatever you need for your game!

## Overview

The whole user properties mechanism relies upon Bevy reflect.

To get things running, you first need to :

- declare the types you want to use as custom properties in your code and make them "reflectable"
- run your game once so these types are exported in a `.json` formated file, readable by Tiled
- import this `.json` file in Tiled editor to make these types available

Once done, you will be able to use these types directly in the Tiled editor and when loading your map, `Component` or `Resource` corresponding to your types will be automatically inserted.

For a quick demonstration, you can have a look to the [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/properties_basic.rs).

## Declare types to be used as custom properties

Your Tiled map, layer, tile or object will be represented by a Bevy `Entity`.
So, it makes sense that if you want to add custom properties to them, these properties should either be a `Component` or a `Bundle`.

In order to be usable in Tiled, your custom types must be "reflectable" :

- your type must derive the `Reflect` trait
- your type must be registered with Bevy

Here's a quick example :

```rust, no_run
use bevy::prelude::*;

// Declare a component and make it "reflectable"
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
struct BiomeInfos {
    block_line_of_sight: bool,
    ty: BiomeType,
}

// Any 'sub-type' which is part of our component must also be "reflectable"
// But it does not have to be a Component
#[derive(Default, Reflect)]
#[reflect(Default)]
enum BiomeType {
    #[default]
    Unknown,
    Forest,
    Plain,
    Moutain,
    Desert,
}

// Finally, register our top-level struct in Bevy registry
fn main() {
    App::new()
        .register_type::<BiomeInfos>();
}
```

And that's all for the code part !
Next time you run your app, this `Component` will be exported in a `.json` export file which can then be imported in Tiled (more on that in next section).

In the above example, our custom type also derive the `Default` trait.
It is particulary useful: this way, you don't have to fill all the fields of your custom type when you use it in Tiled.
We will use the type default value to fill the gaps.

## Import custom properties in Tiled

Before you can add custom properties to your map, you will need to export them from your app then import them in Tiled.

When running with the `user_properties` feature, your app will automatically produce a `.json` file export of all types registered with Bevy.
By default, this file will be produced in your workspace with the name `tiled_types_export.json`.
You can change this file name or even disable its production by tweaking the `TiledMapPlugin` configuration (see [`TiledMapPluginConfig`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/struct.TiledMapPluginConfig.html)).

You can then import this file to Tiled.
To do so, in Tiled, navigate to View -> Custom Types Editor:

![view-custom-types](images/properties_view-types.png)

Click on the `Import` button and load your file:

![import-custom-types](images/properties_import-types.png)

Once it is done, you will be able to see all the custom types that you have imported from your application.
Note that it concerns all the types that derive the `Reflect` trait: there can be quite a lot !

![view-custom-types](images/properties_custom-type.png)

## Add custom properties to your map

Once this setup is done, you can add custom properties to your map.

You must select the element you want to add a property on, right-click in the "Custom Properties" panel, then select "Add Property" :

![add-property](images/properties_add-property.png)

This will display a popup from which you can select the type you want to add to this element and give it a name.

For instance, you can add the `BiomeInfos` type from previous example :

![biome-infos](images/properties_biome-infos.png)

And finally update the values that make sense for this particular element.
When you load the map, this element entity will have the proper `Component` with the value you set here.

You should only add properties imported from Bevy: adding ones that you created only in Tiled will not be loaded in Bevy if they do not contain actual Bevy `Component`s.

## Special considerations

You can add custom properties to different Tiled elements, such as objects, layers or the map itself.
To add properties on tiles, you should edit the tileset itself.
For more information on how to do add custom properties, see the [official Tiled documentation](https://doc.mapeditor.org/en/stable/manual/custom-properties/).

Finally, you are not limited to Bevy `Component`s, you can also add `Resource`s to your map.
Since `Resource`s are not attached to a particular entity and they are shared accros your app, we chose to restrict their usage only as Tiled map properties.
If you add a resource to another Tiled element it will just be ignored.
