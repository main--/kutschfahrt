
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



ITEM_TRIGGERS = {}
def item_trigger(name=None):
    def register(func, name):
        if name is None:
            name = func.__name__.replace('trigger_', '')
        ITEM_TRIGGERS[name] = func
        return func
    return lambda x: register(x, name)


def null_trigger(state, old_owner, new_owner): pass

@item_trigger('koffer_schlüssel')
@item_trigger('koffer_kelch')
def trigger_koffer(state, old_owner, new_owner):
    try:
        item = state['item_stack'].pop()
    except IndexError:
        pass
    else:
        state['players'][old_owner]['items'].append(item)


class Gamestate:
    def __init__(self, state):
        self.state = state
    def num_players(self):
        return len(self.state['seats'])
    def next_player(self, player):
        seats = self.state['seats']
        return seats[seats.index(player) - len(seats) + 1]
    def attack_support_list(self, attacker, defender):
        seats = self.state['seats']
        seats = (seats * 2)[seats.index(attacker):][:len(seats)]
        seats.remove(attacker)
        seats.remove(defender)
        return seats

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
                count += self.count_items(player, 'koffer_schlüssel')
            if item == 'kelch':
                count += self.count_items(player, 'koffer_kelch')
        return count

    def apply_action(self, actor, action):
        turn = self.state['turn']
        ta = turn['action']
        if ta is None:
            # waiting for quickblink
            if turn['player'] != actor:
                # your move -> you play
                raise ValueError('not your turn')

            if action['action'] == 'pass':
                turn['player'] = self.next_player(actor)
            elif action['action'] == 'announce_victory':
                if 'schwarze_perle' in self.state['players'][actor]['items']:
                    raise ValueError()
                teammates = action['other_players']
                turn['player'] = None # game is over
                faction = self.state['players'][actor]['faction']
                if not self.validate_victory(actor, teammates):
                    # swap faction
                    faction = 'orden' if faction == 'bruderschaft' else 'bruderschaft'
                self.state['winner'] = faction
            elif action['action'] == 'swap_item':
                # can only offer items you have
                if action['item'] not in self.state['players'][actor]['items']:
                    raise ValueError()

                turn['action'] = action
                # only the partner will be able to see this action
            elif action['action'] == 'attack':
                action['votes'] = []
                action['buffs'] = []
                action['passed'] = set()
                turn['action'] = action
            else:
                raise ValueError()
        elif ta['action'] == 'swap_item':
            # only the swap partner may react
            if actor != ta['partner']:
                return ValueError()
            offerer = turn['player']
            if action['action'] == 'deny_swap':
                # TODO rache
                turn['player'] = self.next_player(offerer)
                turn['action'] = None
            elif action['action'] == 'accept_swap':
                item1 = ta['item']
                item2 = action['item']
                self.state['players'][offerer]['items'].remove(item1)
                self.state['players'][actor]['items'].append(item1)
                self.state['players'][actor]['items'].remove(item2)
                self.state['players'][offerer]['items'].append(item2)

                # TODO triggers
                ITEM_TRIGGERS.get(item1, null_trigger)(self.state, offerer, actor)
                ITEM_TRIGGERS.get(item2, null_trigger)(self.state, actor, offerer)

                turn['player'] = self.next_player(offerer)
                turn['action'] = None
            else:
                raise ValueError()
        elif ta['action'] == 'attack':
            # TODO priester lol
            attacker = turn['player']
            defender = ta['victim']

            support_list = self.attack_support_list(attacker, defender)
            have_votes = len(ta['votes'])
            votes_complete = have_votes >= len(support_list)
            if action['action'] == 'attack_item_job_pass':
                ta['passed'].add(actor)
                if len(ta['passed']) == self.num_players():
                    # resolve fight
                    # TODO impl phase and stuff
                    turn['player'] = self.next_player(attacker)
                    turn['action'] = None
            elif action['action'] == 'attack_support':
                if votes_complete:
                    raise ValueError()
                relevant_actor = support_list[have_votes]
                if actor != relevant_actor:
                    raise ValueError()
                ta['votes'].append(action['vote'])
            elif action['action'] == 'attack_item_job':
                if not votes_complete:
                    raise ValueError()
                ta['passed'].clear()
                # TODO: apply buffs
            else:
                raise ValueError()
        else:
            raise ValueError()
    def perspective_action(self, player):
        action = self.state['turn']['action']
        if not action:
            return None
        if action['action'] == 'swap_item' and action['partner'] == player:
            return action
        return None

    def perspective(self, player):
        s = self.state
        return {
            'seats': s['seats'],
            'players': { k: v if k == player else { 'items': len(v['items']) } for k,v in s['players'].items() },
            'item_stack': len(s['item_stack']),
            'turn': {
                'player': s['turn']['player'],
                'action': self.perspective_action(player),
            }
        }

TEST_GAMESTATE = {
    'item_stack': ['handschuhe'],
    'seats': ['gundla', 'sarah', 'marie', 'zacharias'],
    'turn': { 'player': 'gundla', 'action': None },
    'players': {
        'gundla': {'faction': 'orden','job': 'hellseher', 'items': ['sextant', 'schlüssel']},
        'sarah': {'faction': 'bruderschaft','job': 'duellant', 'items': ['dolch','peitsche']},
        'marie': {'faction': 'orden', 'job': 'diplomat', 'items': ['schwarze_perle', 'schlüssel', 'koffer_schlüssel']},
        'zacharias': {'faction': 'bruderschaft', 'job': 'leibwächter', 'items': ['schlüssel']},
    }
}

import unittest, copy

class TestKutschfahrt(unittest.TestCase):
    def setUp(self):
        self.gs = Gamestate(copy.deepcopy(TEST_GAMESTATE))

    def test_attack_support_list(self):
        self.assertEqual(self.gs.attack_support_list('sarah', 'marie'), ['zacharias', 'gundla'])
        self.assertEqual(self.gs.attack_support_list('sarah', 'zacharias'), ['marie', 'gundla'])
        self.assertEqual(self.gs.attack_support_list('zacharias', 'sarah'), ['gundla', 'marie'])


    def test_passing(self):
        self.assertEqual(self.gs.state['turn'], { 'player': 'gundla', 'action': None })
        self.gs.apply_action('gundla', {'action': 'pass'})
        self.assertEqual(self.gs.state['turn'], { 'player': 'sarah', 'action': None })
        self.gs.apply_action('sarah', {'action': 'pass'})
        self.assertEqual(self.gs.state['turn'], { 'player': 'marie', 'action': None })
        self.gs.apply_action('marie', {'action': 'pass'})
        self.gs.apply_action('zacharias', {'action': 'pass'})
        self.assertEqual(self.gs.state, TEST_GAMESTATE)
    def test_wrong_player(self):
        with self.assertRaises(ValueError):
            self.gs.apply_action('sarah', {'action': 'pass'})
        with self.assertRaises(ValueError):
            self.gs.apply_action('marie', {'action': 'pass'})
        self.gs.apply_action('gundla', {'action': 'pass'})

    
    def test_announce_victory_blackpearl(self):
        with self.assertRaises(ValueError):
            self.gs.state['turn']['player'] = 'marie'
            self.gs.apply_action('marie', {'action': 'announce_victory', 'other_players': []})
    def test_announce_victory_wrongteam(self):
        self.gs.apply_action('gundla', {'action': 'announce_victory', 'other_players': ['marie', 'zacharias']})
        self.assertEqual(self.gs.state['winner'], 'bruderschaft')
    def test_announce_victory_wrongitems(self):
        self.gs.apply_action('gundla', {'action': 'announce_victory', 'other_players': ['marie']})
        self.assertEqual(self.gs.state['winner'], 'bruderschaft')
    def test_announce_victory_koffer(self):
        self.gs.state['item_stack'] = []
        self.gs.apply_action('gundla', {'action': 'announce_victory', 'other_players': ['marie']})
        self.assertEqual(self.gs.state['winner'], 'orden')
    

    def test_swap_deny(self):
        self.gs.apply_action('gundla', {'action': 'swap_item', 'item': 'schlüssel', 'partner': 'marie'})
        self.gs.apply_action('marie', {'action': 'deny_swap'})
        self.assertEqual(self.gs.state['players'], TEST_GAMESTATE['players'])
        self.assertEqual(self.gs.state['turn'], {'player': 'sarah', 'action': None})
    def test_swap_accept(self):
        self.gs.apply_action('gundla', {'action': 'swap_item', 'item': 'schlüssel', 'partner': 'marie'})
        self.gs.apply_action('marie', {'action': 'accept_swap', 'item': 'schwarze_perle'})
        self.assertNotEqual(self.gs.state['players'], TEST_GAMESTATE['players'])
        self.assertEqual(sorted(self.gs.state['players']['gundla']['items']), sorted(['sextant', 'schwarze_perle']))
        self.assertEqual(sorted(self.gs.state['players']['marie']['items']), sorted(['schlüssel', 'koffer_schlüssel', 'schlüssel']))
        self.assertEqual(self.gs.state['turn'], {'player': 'sarah', 'action': None})
    def test_swap_trigger_koffer(self):
        self.gs.apply_action('gundla', {'action': 'swap_item', 'item': 'schlüssel', 'partner': 'marie'})
        self.gs.apply_action('marie', {'action': 'accept_swap', 'item': 'koffer_schlüssel'})
        self.assertNotEqual(self.gs.state['players'], TEST_GAMESTATE['players'])
        self.assertEqual(sorted(self.gs.state['players']['gundla']['items']), sorted(['sextant', 'koffer_schlüssel']))
        self.assertEqual(sorted(self.gs.state['players']['marie']['items']), sorted(['schlüssel', 'schwarze_perle', 'schlüssel', 'handschuhe']))
        self.assertEqual(self.gs.state['turn'], {'player': 'sarah', 'action': None})
    # TODO: test other triggers


    def test_attack(self):
        self.gs.apply_action('gundla', {'action': 'attack', 'victim': 'marie'})
        self.gs.apply_action('sarah', {'action': 'attack_support', 'vote': 'defend'})
        self.gs.apply_action('zacharias', {'action': 'attack_support', 'vote': 'abstain'})
        self.gs.apply_action('zacharias', {'action': 'attack_item_job_pass'})
        self.gs.apply_action('sarah', {'action': 'attack_item_job', 'item': 'peitsche'})
        self.gs.apply_action('zacharias', {'action': 'attack_item_job', 'job': 'leibwächter'})
        self.gs.apply_action('gundla', {'action': 'attack_item_job_pass'})
        self.gs.apply_action('marie', {'action': 'attack_item_job_pass'})
        self.gs.apply_action('sarah', {'action': 'attack_item_job_pass'})
        self.gs.apply_action('zacharias', {'action': 'attack_item_job_pass'})
        self.assertEqual(self.gs.state['turn'], {'player': 'sarah', 'action': None})


if __name__ == '__main__':
    unittest.main()

