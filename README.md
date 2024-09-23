**ASCII - PacMan**

This project is about making a PacMan game in the terminal.

**__Please Note:__** If you are working under Windows, switch to the Windows console host. Although the game is technically fully functional in the normal Windows terminal, it can run worse than in the Windows console host, especially when it comes to rendering the individual frames, which can lead to a flickering effect in the normal terminal.

**__general information:__**
 - A new frame is rendered every 120ms. ( best speed for PacMan in my opinion )
 - The map can be scaled and changed in every ( for me known ) possible way.
 - W / A / S / D and UP / LEFT / DOWN / RIGHT -Keys work.
 - The game also contains an input queue for smoother / easier movement.

**__problems to be solved:__**
- A big problem in my opinion is that it is not possible for the Pacman to change direction at a position with an odd index in the vector.
- Also, the movement on the y-axis is not as smooth as the movement on the x-axis, because an if statement checks if the current frame is an odd number or not, if yes, the Pac-Man moves, if not, it doesn't move.
 The reason for this is that there are no spaces between the y-coordinates, which in my opinion was not necessary because of the symmetry.

23.09.2024 - (dd/mm/yyyy) -> Update:

  ![image](https://github.com/user-attachments/assets/fd04b19b-3ca9-4b71-99c3-581a58344a9f)


__This is how the game looks like in the current state. (sounds are included)__

__coming soon:__

- enemy ghosts
- power pallets (energizer, power pills, whatever)
- life tracker
- Game Over screen
