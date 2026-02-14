def display_bitboard(bitboard):
    for rank in range(8):
        for file in range(8):
            square_index = rank * 8 + file
            if (bitboard >> square_index) & 1:
                print(' 1', end=' ')
            else:
                print(' ·', end=' ')
        print()

display_bitboard(0x7f7f7f7f7f7f7f7f)
