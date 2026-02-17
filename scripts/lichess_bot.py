from dotenv import load_dotenv
import os
import berserk
import threading
import subprocess
from colorama import init as colorama_init
from colorama import Fore
from colorama import Style
import time
from os.path import dirname, join, abspath
import chess.engine

colorama_init()


load_dotenv()
API_TOKEN = os.getenv("LICHESS_API_KEY")
WORKSPACE_PATH = dirname(dirname(abspath(__file__)))
ENGINE_PATH = [join(WORKSPACE_PATH, "target", "release", "sterm"), "--ai", "simple"]
# ENGINE_PATH = "/usr/bin/stockfish"

print(f"{Fore.LIGHTGREEN_EX}[+] Starting Lichess Bot...{Style.RESET_ALL}")
session = berserk.TokenSession(API_TOKEN)
client = berserk.Client(session=session)

class Game(threading.Thread):
    def __init__(self, client, game_id, initial_fen, player_color='white', username=None):
        super().__init__()
        self.game_id = game_id
        self.client = client
        self.stream = client.bots.stream_game_state(game_id)
        self.current_state = next(self.stream)
        self.username = username
        self.stopped = False
        self.engine = chess.engine.SimpleEngine.popen_uci(ENGINE_PATH)
        self.board = chess.Board(initial_fen)

        print(f"{Fore.GREEN}[+] Engine process started {self.engine}{Style.RESET_ALL}")
        print(f"{Fore.GREEN}[+] Initial FEN sent to engine: {initial_fen}{Style.RESET_ALL}")
        self.player_color = chess.WHITE if player_color.lower() == 'white' else chess.BLACK

        if self.board.turn == self.player_color:
            print(f"{Fore.YELLOW}[!] It's our turn to move!{Style.RESET_ALL}")
            self.generate_next_move()
    
    def run(self):
        for event in self.stream:
            if self.stopped:
                break

            if event['type'] == 'gameState':
                self.handle_state_change(event)
            elif event['type'] == 'chatLine':
                self.handle_chat_line(event)
    
    def generate_next_move(self):
        limit = chess.engine.Limit(time=2.0)
        elem = self.engine.play(self.board, limit=limit)

        if elem.move is None:
            print(f"{Fore.RED}[!] Engine failed to generate a move, resigning...{Style.RESET_ALL}")
            self.client.bots.resign_game(self.game_id)
            return
        else:
            self.client.bots.make_move(self.game_id, elem.move)
            print(f"{Fore.CYAN}[+] Engine recommends move '{elem.move}'{Style.RESET_ALL}")

        # self.subprocess.stdin.flush()
        # outputs = self.subprocess.stdout.readline().strip()
        # if outputs == "":
        #     print(f"{Fore.RED}[!] No output from engine, cannot generate move{Style.RESET_ALL}")
        #     return
        # else:
        #     move, score = outputs.split(',')
        #     score = float(score)
        #     print(f"{Fore.CYAN}[+] Engine recommends move '{move}' with score: {score}{Style.RESET_ALL}")

        #     # self.subprocess.stdin.write(f"move {move}\n")
        #     # self.subprocess.stdin.flush()
        #     self.client.bots.make_move(self.game_id, move)

    def handle_state_change(self, game_state):
        if game_state['status'] != 'started':
            print(f"{Fore.YELLOW}[!] Game {self.game_id} ended with status: {game_state['status']}{Style.RESET_ALL}")
            self.kill()
            return

        move = game_state['moves'].split()[-1]
        print(f"{Fore.BLUE}[GAME STATE] Game {self.game_id} play {move}{Style.RESET_ALL}")
        self.board.push_uci(move)
        
        if self.board.turn == self.player_color:
            self.generate_next_move()

        # Send back the move recommended by the engine
        # move = 'd7d5' # Placeholder move, replace with actual move from engine output
        # self.client.bots.make_move(self.game_id, move)

    def kill(self):
        print(f"{Fore.RED}[!] Killing game {self.game_id} and engine process...{Style.RESET_ALL}")
        self.client.bots.post_message(self.game_id, "Nice game! Thanks for playing :)")
        self.stopped = True
        self.stream.close()
        self.engine.quit()
        # self.subprocess.stdin.write("exit\n")
        # time.sleep(0.5)  # Give the subprocess some time to exit gracefully
        # # self.subprocess.stdin.flush()
        # self.subprocess.terminate()

    def handle_chat_line(self, chat_line):
        print(f"{Fore.MAGENTA}[CHAT] {chat_line['username']}: {chat_line['text']}{Style.RESET_ALL}")
        
        if chat_line['username'] == self.username:
            text = chat_line['text'].lower()

            if "good move" in text:
                self.client.bots.post_message(self.game_id, "Thanks! I try my best.")
            elif "bad move" in text:
                self.client.bots.post_message(self.game_id, "Oh no! I'll try to do better next time.")
            elif text in ["hello", "hi", "hey"]:
                self.client.bots.post_message(self.game_id, f"Hello {chat_line['username']}! Good luck and have fun!")
            elif text == "kill":
                self.kill()



def should_accept(challenge):
    challenger = challenge['challenger']
    challenger_id = challenger['id']
    challender_rating = challenger['rating']
    accepting = challenger_id in ["magelan74", "michelducartier"]
    print(f"{Fore.YELLOW}[!] Received challenge from {challenger_id} (rating: {challender_rating}) ({'accepted' if accepting else 'declined'}){Style.RESET_ALL}")
    return accepting

# Always compile before running it
print(f"{Fore.BLUE}[ ] Compiling `sterm` binary to the latest version{Style.RESET_ALL}")
subprocess.run(["cargo", "build", "--bin", "sterm", "--release"], cwd=WORKSPACE_PATH).check_returncode()

print(f"{Fore.GREEN}[+] Bot is running...{Style.RESET_ALL}")
for event in client.bots.stream_incoming_events():
    if event["type"] == "challenge":
        challenge = event['challenge']
        if should_accept(challenge):
            client.bots.accept_challenge(challenge['id'])
        else:
            client.bots.decline_challenge(challenge['id'])
    elif event['type'] == 'gameStart':
        game_fen = event['game']['fen']
        game_id = event['game']['gameId']
        player_color = event['game']['color'].lower() # 'white' or 'black'
        username = event['game']['opponent']['username']

        print(f"{Fore.GREEN}[+] Game started with ID: {game_id} (playing as {player_color} against {username}){Style.RESET_ALL}")

        game = Game(client, game_id, game_fen, player_color, username)
        game.start()


