# Malcolm's Hallway/Room placement logic ideas

## Room placement
Dependant on which sides of the room doors are located. Rooms should rarely, if ever, be placed in such a way
that a door is very close to or touching a boundary of the building. There should be a certain distance from
each boundary where rooms with doors facing that boundary can't/almost certainly won't be generated.

## Hallway Drawing

- Draw hallways in a straight line from every door on the map in the direction that the door is facing.
- If the tile where the next hallway would be drawn is a door, stop drawing the hallway.
- If the tile where the next hallway would be drawn is an outer boundary, either:
  - Stop, or
  - Change direction
- If the tile where the next hallway would be drawn is another hallway, connect the hallways and stop drawing both.
- If the tile where the next hallway would be drawn is a room boundary that is not a door, change direction.
- If the tile where the next hallway would be drawn crosses over the same X or Y coordinate as another hallway (dependant on whether the hallway is horizontal or vertical), change direction to connect with that hallway.
