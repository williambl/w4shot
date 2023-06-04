use crate::State;
use crate::lose::LoseState;
use crate::sprites::{render_eye, render_powerup, render_ship};
use crate::State::{Game, Lose};
use crate::wasm4::{BUTTON_1, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP, DRAW_COLORS, rect, text, trace};

#[derive(Copy, Clone)]
pub struct GameState {
    player_x: u8,
    player_y: u8,
    player_dx: i8,
    player_dy: i8,
    player_health: u8,
    player_hurt_cooldown: u8,
    time: u32,
    difficulty: u8,
    entity_spawn_interval: u16,
    entities: [Entity; 64],
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Entity {
    x: u8,
    y: u8,
    size: u8,
    dx: i8,
    dy: i8,
    age: u16,
    entity_type: EntityType,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum EntityType {
    None,
    Bullet {
        player: bool
    },
    BasicEnemy {
        seed: u8,
        aims: bool,
    },
    PowerUp,
}

const EMPTY_ENTITY: Entity = Entity {
    x: 0,
    y: 0,
    size: 0,
    dx: 0,
    dy: 0,
    age: 0,
    entity_type: EntityType::None,
};

enum GameEvent {
    PlayerHurt,
    PowerUp
}

struct ChangeRequests<'a> {
    entities_to_add: Vec<Entity>,
    entities_to_remove: Vec<& 'a Entity>,
    events: Vec<GameEvent>,
}

impl GameState {
    pub fn new(difficulty: u8) -> Self {
        let mut state = Self {
            player_x: 80,
            player_y: 100,
            player_dx: 0,
            player_dy: 0,
            player_health: 2,
            player_hurt_cooldown: 0,
            time: 0,
            difficulty,
            entity_spawn_interval: (600u16 - 5u16 * (difficulty as u16).saturating_pow(2)).clamp(1, 600),
            entities: [EMPTY_ENTITY; 64],
        };
        state
    }

    fn add_entity(&mut self, entity: Entity) -> bool {
        for existing_entity in self.entities.iter_mut() {
            if existing_entity.entity_type == EntityType::None {
                *existing_entity = entity;
                return true;
            }
        }
        false
    }

    fn update_movement_from_gamepad(&mut self, gamepad: u8) {
        if gamepad & BUTTON_UP != 0 {
            self.player_dy -= 1;
        }
        if gamepad & BUTTON_DOWN != 0 {
            self.player_dy += 1;
        }
        if gamepad & BUTTON_LEFT != 0 {
            self.player_dx -= 1;
        }
        if gamepad & BUTTON_RIGHT != 0 {
            self.player_dx += 1;
        }
    }

    fn update_player(&mut self, gamepad: u8, last_gamepad: u8) {
        self.player_dx = 0;
        self.player_dy = 0;
        self.player_hurt_cooldown = self.player_hurt_cooldown.saturating_sub(1);

        self.update_movement_from_gamepad(gamepad);
        self.update_movement_from_gamepad(last_gamepad);

        self.player_x = if self.player_dx < 0 {
            self.player_x.saturating_sub(self.player_dx.saturating_abs() as u8)
        } else {
            self.player_x.saturating_add(self.player_dx as u8)
        }.clamp(0, 160);
        self.player_y = if self.player_dy < 0 {
            self.player_y.saturating_sub(self.player_dy.saturating_abs() as u8)
        } else {
            self.player_y.saturating_add(self.player_dy as u8)
        }.clamp(0, 160);

        if gamepad & BUTTON_1 != 0 {
            match self.player_health.min(3) {
                1 => {
                    if self.time % 30 == 0 {
                        self.add_entity(Entity {
                            x: self.player_x,
                            y: self.player_y.saturating_sub(3),
                            size: 1,
                            dx: 0,
                            dy: -3,
                            age: 0,
                            entity_type: EntityType::Bullet { player: true },
                        });
                    }
                }
                2 => {
                    if self.time % 10 == 0 {
                        self.add_entity(Entity {
                            x: self.player_x,
                            y: self.player_y.saturating_sub(3),
                            size: 1,
                            dx: 0,
                            dy: -3,
                            age: 0,
                            entity_type: EntityType::Bullet { player: true },
                        });
                    }
                }
                3 => {
                    if self.time % 10 == 0 {
                        self.add_entity(Entity {
                            x: self.player_x,
                            y: self.player_y.saturating_sub(3),
                            size: 1,
                            dx: 0,
                            dy: -3,
                            age: 0,
                            entity_type: EntityType::Bullet { player: true },
                        });
                        self.add_entity(Entity {
                            x: self.player_x,
                            y: self.player_y.saturating_sub(3),
                            size: 1,
                            dx: -1,
                            dy: -3,
                            age: 0,
                            entity_type: EntityType::Bullet { player: true },
                        });
                        self.add_entity(Entity {
                            x: self.player_x,
                            y: self.player_y.saturating_sub(3),
                            size: 1,
                            dx: 1,
                            dy: -3,
                            age: 0,
                            entity_type: EntityType::Bullet { player: true },
                        });
                    }
                }
                _ => {}
            }
        }
    }

    fn spawn_new_entities(&mut self) {
        if self.time % self.entity_spawn_interval as u32 == 0 {
            let mut random = self.get_random();
            let enemy_count = (random % 6u32) as u8 + (6f32 * self.difficulty as f32 / 10f32) as u8;
            let x_increment = 160u8/enemy_count;
            for i in 0..enemy_count {
                self.add_entity(Entity {
                    x: i * x_increment,
                    y: 10,
                    size: 8,
                    dx: 0,
                    dy: 0,
                    age: 0,
                    entity_type: EntityType::BasicEnemy { seed: random as u8, aims: self.difficulty > 6 },
                });
                random = next_random(random);
            }
            if self.time % 600u32 == 0 {
                self.add_entity(Entity {
                    x: (random as u8 % 140u8) + 10u8,
                    y: ((random >> 8) as u8 % 100u8) + 10u8,
                    size: 8,
                    dx: ((random >> 16) as u8 % 3u8) as i8 - 1,
                    dy: ((random >> 24) as u8 % 3u8) as i8 - 1,
                    age: 0,
                    entity_type: EntityType::PowerUp,
                });
            }
        }
    }

    fn with_updated_entities(&self) -> GameState {
        let self_clone = &self.clone();
        let mut new_state = self.clone();
        let new_entities_and_change_requests: Vec<(Entity, ChangeRequests)> = new_state.entities.iter().map(|entity| entity.update(self_clone)).collect();
        let mut new_entities: Vec<Entity> = Vec::new();
        let mut change_requests: Vec<ChangeRequests> = Vec::new();
        for (entity, change_request) in new_entities_and_change_requests {
            new_entities.push(entity);
            change_requests.push(change_request);
        }
        new_state.entities = <[Entity; 64]>::try_from(new_entities).unwrap();

        for ChangeRequests { entities_to_add, entities_to_remove, events } in change_requests {
            for entity in entities_to_remove {
                for existing_entity in new_state.entities.iter_mut() {
                    if existing_entity == entity {
                        *existing_entity = EMPTY_ENTITY;
                        break;
                    }
                }
            }
            for entity in entities_to_add {
                new_state.add_entity(entity);
            }
            for event in events {
                match event {
                    GameEvent::PlayerHurt => {
                        if new_state.player_hurt_cooldown == 0 {
                            new_state.player_health = new_state.player_health.saturating_sub(1);
                            new_state.player_hurt_cooldown = 90;
                        }
                    },
                    GameEvent::PowerUp => new_state.player_health = new_state.player_health.saturating_add(1)
                };
            }
        }
        new_state
    }

    fn get_random(self) -> u32 {
        next_random(self.time)
    }
}

fn next_random(seed: u32) -> u32 {
    (seed * 15417) ^ (seed << 31) ^ ((seed * 123651) >> 7)
}

fn entity_collides_with_wall(entity: &Entity) -> bool {
    entity.x == 0 && entity.dx < 0 || entity.x == 160 && entity.dx > 0 || entity.y == 0 && entity.dy < 0 || entity.y == 160 && entity.dy > 0
}

fn collides(entity: &Entity, other_entity: &Entity) -> bool {
    entity.entity_type != EntityType::None && other_entity.entity_type != EntityType::None
    && entity.x.saturating_sub(entity.size/2) < other_entity.x.saturating_add(other_entity.size/2)
    && entity.x.saturating_add(entity.size/2) > other_entity.x.saturating_sub(other_entity.size/2)
    && entity.y.saturating_sub(entity.size/2) < other_entity.y.saturating_add(other_entity.size/2)
    && entity.y.saturating_add(entity.size/2) > other_entity.y.saturating_sub(other_entity.size/2)
}

fn collides_with_player(entity: &Entity, state: &GameState) -> bool {
    entity.entity_type != EntityType::None
        && entity.x.saturating_sub(entity.size/2) < state.player_x.saturating_add(4)
        && entity.x.saturating_add(entity.size/2) > state.player_x.saturating_sub(4)
        && entity.y.saturating_sub(entity.size/2) < state.player_y.saturating_add(4)
        && entity.y.saturating_add(entity.size/2) > state.player_y.saturating_sub(4)
}

impl Entity {
    fn update_movement(&mut self) {
        self.x = if self.dx < 0 {
            self.x.saturating_sub(self.dx.saturating_abs() as u8)
        } else {
            self.x.saturating_add(self.dx as u8)
        }.clamp(0, 160);
        self.y = if self.dy < 0 {
            self.y.saturating_sub(self.dy.saturating_abs() as u8)
        } else {
            self.y.saturating_add(self.dy as u8)
        }.clamp(0, 160);
    }

    fn update(self, state_snapshot: &GameState) -> (Entity, ChangeRequests) {
        let mut change_requests = ChangeRequests {
            entities_to_add: Vec::new(),
            entities_to_remove: Vec::new(),
            events: Vec::new(),
        };
        let mut new_entity = self.clone();
        new_entity.age += 1;
        match new_entity.entity_type {
            EntityType::None => {},
            EntityType::Bullet {player} => {
                new_entity.update_movement();
                if new_entity.age > 200 || entity_collides_with_wall(&new_entity) {
                    new_entity = EMPTY_ENTITY.clone();
                }
                if !player && collides_with_player(&new_entity, state_snapshot) {
                    new_entity = EMPTY_ENTITY.clone();
                    change_requests.events.push(GameEvent::PlayerHurt);
                }
            },
            EntityType::BasicEnemy { seed, aims } => {
                let random = next_random(state_snapshot.get_random() ^ (new_entity.x as u32 * 651) ^ (new_entity.y as u32 * 474));
                if (seed as u16 + new_entity.age) % 60 == 0 {
                    new_entity.dx = if random & 0x10 != 0 { 1 } else { -1 };
                    new_entity.dy = if random & 0x01 != 0 { 1 } else { -1 };
                } else if (seed as u16 + new_entity.age) % 60 == 30 || entity_collides_with_wall(&new_entity) {
                    new_entity.dx = 0;
                    new_entity.dy = 0;
                }
                new_entity.update_movement();
                if (seed as u16 + new_entity.age) % 60 == 0 {
                    let aim_x = if aims { state_snapshot.player_x as f32 - new_entity.x as f32 } else { 0f32 };
                    let aim_y = if aims { state_snapshot.player_y as f32 - new_entity.y as f32 } else { 1f32 };
                    let aim_length = (aim_x as f32 * aim_x as f32 + aim_y as f32 * aim_y as f32).sqrt();
                    let dx = if aim_length == 0f32 { 0i8 } else { (2f32 * aim_x / aim_length) as i8 };
                    let dy = if aim_length == 0f32 { 0i8 } else { (2f32 * aim_y / aim_length) as i8 };
                    change_requests.entities_to_add.push(Entity {
                        x: new_entity.x,
                        y: new_entity.y,
                        size: 1,
                        dx,
                        dy,
                        age: 0,
                        entity_type: EntityType::Bullet { player: false },
                    });
                    if collides_with_player(&new_entity, state_snapshot) {
                        change_requests.events.push(GameEvent::PlayerHurt);
                    }
                }
                for entity in state_snapshot.entities.iter() {
                    if (entity.entity_type == EntityType::Bullet { player: true }) && collides(&new_entity, entity) {
                        new_entity = EMPTY_ENTITY.clone();
                        change_requests.entities_to_remove.push(entity);
                        break;
                    }
                }
            },
            EntityType::PowerUp => {
                if new_entity.x == 0 && new_entity.dx < 0 || new_entity.x == 160 && new_entity.dx > 0 {
                    new_entity.dx = -new_entity.dx;
                }
                if new_entity.y == 0 && new_entity.dy < 0 || new_entity.y == 160 && new_entity.dy > 0 {
                    new_entity.dy = -new_entity.dy;
                }
                new_entity.update_movement();
                if new_entity.age > 900 {
                    new_entity = EMPTY_ENTITY.clone();
                }
                if collides_with_player(&new_entity, state_snapshot) {
                    new_entity = EMPTY_ENTITY.clone();
                    change_requests.events.push(GameEvent::PowerUp);
                }
            },
        }
        (new_entity, change_requests)
    }
}

pub fn update_game(state: GameState, gamepad: u8, last_gamepad: u8) -> State {
    let mut new_state = state;
    new_state.spawn_new_entities();
    new_state.time += 1;
    new_state.update_player(gamepad, last_gamepad);
    new_state = new_state.with_updated_entities();

    if new_state.player_health == 0 {
        Lose(LoseState::new(new_state.time))
    } else {
        Game(new_state)
    }
}

fn render_entities(state: GameState) {
    state.entities.iter().for_each(|entity| match entity.entity_type {
        EntityType::None => { },
        EntityType::Bullet {..} => {
            unsafe { *DRAW_COLORS = 0x0004 }
            // draw rect of size entity.size
            let half_size = (entity.size / 2) as i32;
            rect((entity.x as i32 - half_size), (entity.y as i32 - half_size), entity.size as u32, entity.size as u32);
        },
        EntityType::BasicEnemy {..} => {
            unsafe { *DRAW_COLORS = 0x0432 }
            let half_size = (entity.size / 2) as i32;
            render_eye((entity.x as i32 - half_size), (entity.y as i32 - half_size));
        },
        EntityType::PowerUp => {
            unsafe { *DRAW_COLORS = 0x0432 }
            let half_size = (entity.size / 2) as i32;
            render_powerup((entity.x as i32 - half_size), (entity.y as i32 - half_size));
        },
    });
}

pub fn render_game(state: GameState) {
    render_entities(state);
    unsafe { *DRAW_COLORS = 0x2430 }
    if state.player_hurt_cooldown % 2 == 0 {
        render_ship((state.player_x as i32 - 4), (state.player_y as i32 - 4));
    }
    text(format!("Health: {}", state.player_health).as_str(), 0, 0);
}
