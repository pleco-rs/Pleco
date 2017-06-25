

pub struct UndoMove {

}

pub struct PreUndoMove {
    u8: castling

}
// 0000 0000 0000
// ^ ----> What Piece Moved?
//      ^ ---->

// 32 bits:

// src : 6 bits
// dst : 6 bits
// promo: 1 bit
// Capture: 1 bit

// 0000  ===> Pawn Move
// 0001  ===> Knight Move
// 0010  ===> Bishop Move
// 0011  ===> Rook Move
// 0100  ===> Queen Move
// 0101  ===> King Move
// 0110  ===> Castle Kingside
// 0111  ===> Castle Queenside

// 0000  ===> Captured Pawn
// 0001  ===> Captured Knight
// 0010  ===> Captured Bishop
// 0011  ===> Captured Rook
// 0100  ===> Captured Queen
// 0101  ===> Captured Pawn
// 0110  ===>
// 0111  ===>

// 4 bits: castling
// 0000WWBB, left = 1 -> king side castle available, right = 1 -> queen side castle available


// Capture or Not Capture
// Not Capture -> Castle
//              Castle -> queen Side, Rook side

// Capture ->
//      Ep Capture or

// Promotion or not promotion

// Not Mutually Exlusive: Promotion and Capture


// 0000  ===> Quiet move
// 0001  ===> Double Pawn Push
// 0010  ===> King Castle
// 0011  ===> Queen Castle
// 0100  ===> Capture
// 0101  ===> EP Capture
// 0110  ===>
// 0111  ===>
// 1000  ===> Knight Promotion
// 1001  ===> Bishop Promo
// 1010  ===> Rook   Promo
// 1011  ===> Queen  Capture  Promo
// 1100  ===> Knight Capture  Promotion
// 1101  ===> Bishop Capture  Promo
// 1110  ===> Rook   Capture  Promo
// 1111  ===> Queen  Capture  Promo