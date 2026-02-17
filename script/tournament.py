import chess
import chess.engine
import chess.pgn
import io
import random
import csv
import tqdm
import glob
from os.path import join, dirname, abspath
from colorama import init as colorama_init
from colorama import Fore
from colorama import Style
import logging
import subprocess
import shutil

# logging.basicConfig(level=logging.DEBUG)


OPENING_BOOK_PATH = join(dirname(abspath(__file__)), 'data', 'openings')
colorama_init()

def display_san(moves):
    board = chess.Board()
    san_moves = []
    for move in moves[:-1]:
        san_moves.append(board.san(move))
        board.push(move)
    return board.san(moves[-1]) if moves else ""

def play_game(board, engine1, engine2, time_limit=0.5):
    initial_stack_len = len(board.move_stack)
    displayed_stack_len = 0

    while not board.is_game_over():
        if board.turn == chess.WHITE:
            result = engine1.play(board, chess.engine.Limit(time=time_limit))
        else:
            result = engine2.play(board, chess.engine.Limit(time=time_limit))

        while displayed_stack_len < len(board.move_stack) // 2:
            mv0 = board.move_stack[displayed_stack_len]
            mv0_san = display_san(board.move_stack[:displayed_stack_len + 1])
            mv0_c = Fore.LIGHTGREEN_EX if displayed_stack_len < initial_stack_len else Fore.LIGHTBLACK_EX
            displayed_stack_len += 1

            mv1 = board.move_stack[displayed_stack_len]
            mv1_san = display_san(board.move_stack[:displayed_stack_len + 1])
            mv1_c = Fore.LIGHTGREEN_EX if displayed_stack_len < initial_stack_len else Fore.LIGHTBLACK_EX
            displayed_stack_len += 1

            tqdm.tqdm.write(f"    {mv0_c}{board.fullmove_number // 2}. {mv0_san}{Fore.RESET} {mv1_c}{mv1_san}{Fore.RESET}", end="\n")
        board.push(result.move)
    
    game = chess.pgn.Game.from_board(board)
    game.headers["Result"] = board.result()

    tqdm.tqdm.write(f"{Fore.CYAN}{game}{Style.RESET_ALL}")

    return board.result()

def run_tournament(engine1, engine2, time_limit=0.05, limit_game=None):
    # First read all of the openings from the CSV file
    openings = []
    for file in glob.glob(join(OPENING_BOOK_PATH, '*.tsv')):
        with open(file, 'r') as f:
            reader = csv.reader(f, delimiter='\t')
            next(reader)  # Skip header
            openings += [row for row in reader]

    # Run the tournament
    if limit_game is not None:
        openings = openings[:limit_game]

    bar = tqdm.tqdm(openings, desc="Playing games", unit="game")
    for _, name, pgn in bar:
        # Should always be (<id>. <first move> <second move>)* \*
        moves = []
        for idx, element in enumerate(pgn.split()):
            if element == '*':
                break
            if idx % 3 != 0:
                moves.append(element)
        
        board = chess.Board()
        for mv in moves:
            board.push_san(mv)
        
        # Update the progress bar with the current opening name
        # bar.set_postfix_str(f"Playing opening: {name}")

        # Play the game and get the result
        tqdm.tqdm.write(f"{Fore.LIGHTGREEN_EX}[ ] Playing opening: {name}{Style.RESET_ALL}")
        result = play_game(board, engine1, engine2, time_limit)
        tqdm.tqdm.write(f"{Fore.LIGHTGREEN_EX}    Finished opening: {name} with result {result}{Style.RESET_ALL}")


DEBUG = False

build_args = []
if not DEBUG:
    build_args += ["--release"]
    
# Build the engine and get the path to the executable
WORKSPACE_PATH = dirname(dirname(abspath(__file__)))
ENGINE_PATH = [join(WORKSPACE_PATH, "target", "debug" if DEBUG else "release", "sterm"), "--ai", "simple"]
subprocess.run(["cargo", "build", "--bin", "sterm", *build_args], cwd=WORKSPACE_PATH).check_returncode()

# engine = chess.engine.SimpleEngine.popen_uci("/usr/bin/stockfish")
engine2 = chess.engine.SimpleEngine.popen_uci(ENGINE_PATH)
run_tournament(engine2, engine2)
