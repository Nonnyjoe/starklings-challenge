// dict3.cairo
// Custom data structure using dicts
// Using Felt252Dict in structs allow us to simulate mutable data structures
// In this exercise we have a struct Team where a Felt252Dict maps the name of a player to its level and keeps track of
// the number of player.
// Using the methods set and get from the Felt252DictTrait, implement the required functions to interact with the team
// Make me compile and pass the test!
// Execute `starklings hint dict3` or use the `hint` watch subcommand for a hint.




#[derive(Destruct)]
struct Team {
  level: Felt252Dict<usize>,
  players_count: usize 
}

#[generate_trait]
impl TeamImpl of TeamTrait {
    fn new() -> Team {
        //TODO : initialize empty team with 0 player
        let mut dict: Felt252Dict<u32> = Default::default();
        let new_team = Team{level : dict, players_count : 0};
        new_team
    }

    fn get_level(ref self: Team, name: felt252) -> usize {
        //TODO 
        self.level.get(name)
    }

    fn add_player(ref self: Team, name: felt252, level: usize) -> () {
        //TODO
        self.level.insert(name, level);
        self.players_count = self.players_count + 1;

    }

    fn level_up(ref self: Team, name: felt252) {
        //TODO
        let res = self.level.get(name);
        self.level.insert(name, res + 1);
    }

    fn players_count(self: Team) -> usize {
        //TODO
        self.players_count
    }
}



#[test]
#[available_gas(200000)]
fn test_add_player() {

    let mut team = TeamTrait::new();
    team.add_player('bob', 10);
    team.add_player('alice', 20);

    assert(team.players_count == 2, 'Wrong number of player');
    assert(team.get_level('bob') == 10, 'Wrong level');
    assert(team.get_level('alice') == 20, 'Wrong level');
}

#[test]
#[available_gas(200000)]
fn test_level_up() {

    let mut team = TeamTrait::new();
    team.add_player('bobby',10);
    team.level_up('bobby');

    assert(team.level.get('bobby') == 11, 'Wrong level');
}
