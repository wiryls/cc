# [Project Cube Collection](https://wiryls.github.io/cube-collection/)

This is a simple puzzle game based on [Bevy Engine](https://github.com/bevyengine/bevy).

**Try to move cubes to all target points!**

![level-preview](./docs/images/level-preview.gif)

## Tutorial

Use:

- `Arrow Keys` to move,
- `R` as restart,
- `N` to skip this level,
- `ESC` to reset the game.

## Rules

- You control all green cubes.
- Move cubes to targets to enter the next level.
- Cubes may absorb each others.
   - Red + Green -> Red + Red
   - Green + Blue -> Green + Green
   - Blue + Red -> Blue + Blue
- Cubes in the same color merge when hitting each other.

## Known issues

- [Jitters](https://github.com/bevyengine/bevy/issues/4669) happend while moving cubes.