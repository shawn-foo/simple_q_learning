use core::panic;
use std::{collections::HashMap, env, fmt::format, hash::Hash, str::EncodeUtf16, vec};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use serde::{Serialize, Deserialize};
use std::fs;


fn main() {
    let f = fs::read_to_string("./maze.ron").unwrap();
    let x: Environment = ron::from_str(&f).unwrap();
    let environment = x.clone();
    environment.print_maze();
    let mut q_table = Qtable::new_empty();
    q_table.init_table(environment.get_maze(), &environment);
    q_table.train(environment.clone(), 100, true);
    q_table.print_result(environment.clone());
    println!("{:#?}", environment);
    let t = fs::read_to_string("./test.txt").unwrap();
    println!("{}", t)
}
pub const DISCOUNT_FACTOR: f32 = 0.9;
pub const LEARNING_RATE: f32 = 0.1;
//maze 0=empty 2 = reward 1 = startpoint
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Environment{
    maze: Vec<Vec<i32>>,
    endpoint: Position,
    startpoint: Position
}
// impl Default for Environment{
//     fn default() -> Self {
//         Self { 
//             maze: vec![
//                 vec![2, 0, 0, 0, 0],
//                 vec![0, 0, 0, 0, 0],
//                 vec![0, 0, 0, 0, 0],
//                 vec![0, 0, 0, 0, 0],
//                 vec![0, 0, 0, 1, 0],
//             ].into(),
//             endpoint: Position::new(0, 0), 
//             startpoint: Position::new(3, 4),
//         }
//     }
// }
impl Environment{
    fn get_maze(&self) -> Vec<Vec<i32>>{
        self.maze.clone()
    }
    fn get_type(&self, position: Position) -> i32{
        self.maze[position.y as usize][position.x as usize]
    }
    fn print_maze(&self){
        let mut string = String::from("[\n");
        for (y, x_vector) in self.maze.iter().enumerate(){
            string = string +  "[";
            for (x, value) in x_vector.iter().enumerate(){
                string += format!("{}, ", value).as_str();
            }
            string += "],\n"
        }
        string += "]";
        println!("{}", string);
    }
}

pub const DIRECTION_POSSIBILITIES: i32 = 4;
#[derive(PartialEq, Eq, Hash, Clone, EnumIter, Debug)] 
enum Actions{
    UP,
    DOWN,
    LEFT,
    RIGHT
}
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
struct Position{
    pub x: i32,
    pub y: i32
}
impl Position{
    fn new(x: i32, y: i32) -> Self{
        Self { x, y }
    }
}
#[derive(PartialEq, Eq, Hash, Clone, Debug)] 
struct State{
    pub player_pos: Position
}
impl State{
    fn new(position: Position) -> Self{
        Self { player_pos: position }
    }
    fn check_if_inside(&self, maze: Vec<Vec<i32>>) -> bool {
        let max_y = maze.len() - 1;
        let max_x = maze[0].len() - 1;
        if self.player_pos.x <= max_x as i32 && self.player_pos.y <= max_y as i32 && self.player_pos.x >= 0 && self.player_pos.y >= 0 {
            return true;
        }
        return false;
    }
}
#[derive(Clone)]
struct Qtable{
    values: HashMap<(State, Actions), f32>
}
impl Qtable{
    fn new_empty() -> Self{
        Self { values: HashMap::new() }
    }
    fn init_table(&mut self, maze: Vec<Vec<i32>>, environment: &Environment) {
        for (y, x_vector) in maze.iter().enumerate(){
            for (x, value) in x_vector.iter().enumerate(){
                for direction in Actions::iter(){
                    let new_state = action_and_state_to_new_state(direction.clone(), State::new(Position::new(x as i32, y as i32)));
                    if new_state.check_if_inside(environment.maze.clone()){
                        self.values.insert((State::new(Position::new(x as i32, y as i32)), direction), 0.0);
                    
                    }
                    
                }
            }
        }
    }
    fn get_max_q_value_and_action(&self, state: State, environment: Environment) -> (f32, Actions){
        let mut q_values: Vec<(Actions, f32)> = vec![];

        for current_action in Actions::iter(){
            let new_state = action_and_state_to_new_state(current_action.clone(), state.clone());
            if new_state.check_if_inside(environment.maze.clone()){
                let q_value = self.values.get(&(state.clone(), current_action.clone())).unwrap();
                q_values.push((current_action.clone(), q_value.clone()));
            }
        }
        let mut max_q_value = q_values[0].1;
        let mut max_action = q_values[0].0.clone();
        for (action, value) in q_values.iter(){
            if value > &max_q_value{
                max_q_value = value.clone();
                max_action = action.clone();
            }
        }
        // println!("states and actions: {}", self.values.len());
        return (max_q_value.clone(), max_action);
    }
    fn train(&mut self, environment: Environment, learning_cycles: i32, print: bool){
        let q_table = self.clone();
        for i in 0..learning_cycles{
            for ((state, action), mut q_value) in self.values.iter_mut(){
                let max_q_value = q_table.get_max_q_value_and_action(state.clone(), environment.clone()).0;
                let new_q_value =  ((1 as f32 - LEARNING_RATE) * q_value.clone() + LEARNING_RATE * (cal_reward(environment.clone(), action_and_state_to_new_state(action.clone(), state.clone())) + DISCOUNT_FACTOR * max_q_value));
                *q_value = new_q_value;
                println!("q value {}", q_value.clone());
            }
            if print == true{
                println!("transing cycle{} complete", i + 1)
            }

        }

    }
    fn print_result(&self, environment: Environment) {
        let mut state = State::new(environment.startpoint.clone());
        let mut actions: Vec<char> = vec![];
        let mut reached_goal = false;
        while reached_goal == false{
            let (q_value, action) = self.get_max_q_value_and_action(state.clone(), environment.clone());
            match action {
                Actions::DOWN => {
                    actions.push('↓');
                    println!("↓")
                }
                Actions::UP => {
                    actions.push('↑');
                    println!("↑")
                }
                Actions::LEFT => {
                    actions.push('←');
                    println!("←")
                }
                Actions::RIGHT => {
                    actions.push('→');
                    println!("→")
                }
            }
            let maze_value = environment.maze[state.player_pos.y as usize][state.player_pos.x as usize];
            state = action_and_state_to_new_state(action, state.clone());
            if maze_value == 2 || state.player_pos == environment.endpoint{
                reached_goal = true;
            }
            
            
        }
        print!("moves{:#?}", actions);
    }

}

//functions
fn cal_reward(environment: Environment, state: State) -> f32{
    let mut reward:f32 = 0.0;
    let endpoint = environment.endpoint;
    let current_pos = environment.maze[state.player_pos.y as usize][state.player_pos.x as usize];
    if current_pos == 2{
        reward += 10000000000.0
    }
    else if current_pos == 1 {
        reward -= 100000000.0
    }
    else if current_pos == 0{

        //implememtn out of bounds stuff
        if state.player_pos.x > environment.maze[0].len() as i32 -1{
            reward -= 10000.0;
        }
        else if state.player_pos.x > environment.maze.len() as i32 -1 {
            reward -= 10000.0;
        } 
        else {
            reward += 1000000.0;
            //find the difference
            let distance = (((state.player_pos.x - environment.endpoint.x).pow(2) + (state.player_pos.y - environment.endpoint.y).pow(2)) as f32).sqrt();
            reward += distance * 10.0 * -1.0
        }
    }
    else {
        panic!();
    }
    println!("{}", reward);
    
    reward
    
}
fn action_and_state_to_new_state(action: Actions, old_state: State) -> State{
    let mut state = old_state;
    match action{
        Actions::DOWN => {
            state.player_pos.y += 1;
        },
        Actions::UP => {
            state.player_pos.y -= 1;
        }
        Actions::LEFT => {
            state.player_pos.x -= 1;
        }
        Actions::RIGHT => {
            state.player_pos.x += 1;
        }
    }

    state
}