# Building Blocks

Experimental "block" game engine.

## Features

### Volumetric

The "world" is composed of several block clusters, inspired by [Sable sub-levels](https://modrinth.com/mod/sable). Clusters can be constructed, edited, serialized and deserialized at run time.

Clusters of blocks can be queried for visuals or collision-simulated all at once in a world. The physics calculations are done with [Rapier](https://crates.io/crates/rapier3d), and the special "block-to-block" interactions are inspired by [Create](https://modrinth.com/mod/create) and its addon [Create: Simulated/Aeronautics](https://modrinth.com/mod/create-aeronautics);

### Data-driven

Blocks and their textures, sounds, behaviours are data-driven and scriptable.

## Roadmap & Inspirations

### From Sable

- [x] Several block clusters (like sub-levels);
- [ ] Per-cluster physics (using Rapier);
    - [ ] Cluster-to-cluster joints;

### From Voxy

- [ ] Really good LODs for large worlds;
    - [ ] Static LOD clusters that are always low-detail for things in the distance you do never visit;
    - [ ] LOD clusters that can be easily reified when zoomed in;

### From Immersive Portals / Portal Mod

- [ ] Portals;

### From Axiom & Cozy Building Games

- [ ] Block brushes;
- [ ] Context aware tools (extrude connected blocks like walls, furniture, carpets, etc, with gizmos);

### From Picotron

- [ ] Scripting language based pixels layers for 2D and GUI rendering;

### Other

- [X] Fully data driven blocks;
    - [ ] Complex procedural materials;
    - [ ] Complex (non-cuboid) models;
    - [ ] Collisions and physics properties either per block or per box in case of non cuboid models;
- [ ] Particles;

### Experimental ideas