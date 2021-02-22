from flask import Flask

app = Flask(__name__)

gamestate = {
    'players': {
        'gundla': {
            'faction': 'bruderschaft',
            'job': 'hellseher',
            'items': [
                'sextant',
                'koffer (schlüssel)'
            ]
        }
    },
    'seats': ['gundla', 'sarah', 'marie'],
    ''
    'turn': {
        'player': 'gundla',
        'action': {
            'move': 'attack',
            'target': 'sarah',

            'supporters': {
                'asd': 'attack',
                'asdf': 'defend',
                'asdfg': 'abstain'
            },

            'items': {
            },
            'passed_items_and_jobs': {
                'gundla'
            }
        }
    }
}

command = {
    'kind': 'move',
    'move': 'pass'
}


game = [
    {
        'kind': 'initial',
        'stack': ['abcdefg'],
    }
]



class Gamestate:
    def __init__(self, state):
        self.state = state
    def validate_action(self, action):
        pass
    def next_player(self, player):
        seats = self.state['seats']
        return seats[seats.index(player) - len(seats) + 1]
    def validate_victory(self, announcer, teammates):
        victory_team = self.state['players'][announcer]['faction']
        victory_item = 'schlüssel' if victory_team == 'orden' else 'kelch'
        total_items = 0
        for p in [announcer] + teammates:
            if self.state['players'][p]['faction'] != victory_team:
                return False # lol
            c = self.count_items(p, victory_item)
            if c == 0:
                return False
            total_items += c
        return total_items >= 3 # TODO: ungerade zahl und so
    def count_items(self, player, item):
        count = self.state['players'][player]['items'].count(item)
        if not self.state['item_stack']:
            if item == 'schlüssel':
                count += self.count_items(player, 'koffer_schlüsseü')
            if item == 'kelch':
                count += self.count_items(player, 'koffer_kelch')
        return count

    def apply_action(self, actor, action):
        turn = self.state['turn']
        if turn['action'] is None:
            # waiting for quickblink
            if turn['player'] != actor:
                # your move -> you play
                raise ValueError('not your turn')

            if action['action'] == 'pass':
                turn['player'] = self.next_player(actor)
            elif action['action'] == 'announce_victory':
                teammates = action['other_players']
                turn['player'] = None # game is over
                faction = self.state['players'][actor]['faction']
                if not self.validate_victory(actor, teammates):
                    # swap faction
                    faction = 'orden' if faction == 'bruderschaft' else 'bruderschaft'
                self.state['winner'] = faction
            else:
                raise ValueError()
    def perspective(self, player):
        # return turn, seats verbatim
        # for all OTHER players, remove all secret information except for the number of items
        # for item_stack, return only the count
        state = self.state.copy()
        state['players'] = { k: v if k == player else { 'items': len(v['items']) } for k,v in state['players'].items() }
        state['item_stack'] = len(state['item_stack'])
        return state

# s = { 'item_stack': [], 'seats': ['gundla', 'sarah', 'marie'], 'turn': { 'player': 'gundla', 'action': None }, 'players': { 'gundla': {'faction': 'orden','job':'hellseher','items':['sextant']}, 'sarah': {'faction':'bruderschaft','job':'duellant','items':['dolch','peitsche']}, 'marie': {'faction':'orden', 'job': 'diplomat', 'items':['schwarze_perle']} } }

import unittest

class TestStringMethods(unittest.TestCase):
    def test_upper(self):
        self.assertEqual('foo'.upper(), 'FOO')

if __name__ == '__main__':
    unittest.main()


@app.route("/")
def hello():
    global state
    state += 1
    return str(state)
