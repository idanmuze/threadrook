# Chess Move Notation

![License: MIT](https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExYjFjYWIxY2JlZjFhMTBmNzQ3MTAzNWVjNzdiMWFhNTc1MDQ2M2I4ZSZjdD1n/jYwYzUA37adglQPRAK/giphy.gif)

The chess move format is 'Source Square, Destination Square, (Promo Piece).

e.g. Moving a Queen from A1 to B8 will stringify to `a1b8`.

If there is a pawn promotion involved, the piece promoted to will be appended to the end of the string, alike `a7a8q` in the case of a queen promotion.

Capital Letters represent white pieces, while lower case represents black pieces.
