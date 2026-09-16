








#[derive(Clone)]
pub struct MeleeAttackSpec {
    pub pen_rating: i32,
    pub damage_die: String,
    pub technique_rating: i32
}

pub fn map_penetration(rating: i32) -> [f32; 4] {
    match rating {
        -3 => [0.125, 0.0, 0.0, 0.0],
        -2 => [0.25, 0.01, 0.0, 0.0],
        -1 => [0.5, 0.05, 0.0, 0.0],
        0 => [0.75, 0.1, 0.01, 0.0],
        1 => [0.8, 0.25, 0.05, 0.0],
        2 => [0.9, 0.4, 0.1, 0.005],
        3 => [0.95, 0.5, 0.2, 0.01],
        4 => [0.96, 0.6, 0.3, 0.05],
        5 => [0.98, 0.7, 0.4, 0.1],
        6 => [0.99, 0.8, 0.5, 0.2],
        7 => [0.999, 0.85, 0.55, 0.25],
        x if x > 7 => [1.0, 0.875, 0.6, 0.3],
        _ => [0.1, 0.0, 0.0, 0.0]
    }
}


pub fn describe_hit(pens: usize, damage: i32) -> String {
    match pens {
        4 => {
            if damage > 64 {
                "A vicious assault!".to_string()
            } else if damage > 48 {
                "An intense assault!".to_string()
            } else if damage > 32 {
                "A strong assault!".to_string()
            } else if damage > 16 {
                "A successful assault.".to_string()
            } else {
                "A basic assault.".to_string()
            }
        },
        3 => {
            if damage > 64 {
                "A cataclysmic impact!".to_string()
            } else if damage > 48 {
                "A heavy impact!".to_string()
            } else if damage > 32 {
                "A good impact!".to_string()
            } else if damage > 16 {
                "A solid impact.".to_string()
            } else {
                "An impact.".to_string()
            }
        },
        2 => {
            if damage > 48 {
                "A deadly strike!".to_string()
            } else if damage > 32 {
                "An excellent strike!".to_string()
            } else if damage > 16 {
                "A powerful strike!".to_string()
            } else if damage > 8 {
                "An average strike.".to_string()
            } else {
                "A strike.".to_string()
            }
        },
        1 => {
            if damage > 32 {
                "A brutal hit!".to_string()
            } else if damage > 16 {
                "A forceful hit!".to_string()
            } else if damage > 8 {
                "A solid hit!".to_string()
            } else if damage > 1 {
                "A hit.".to_string()
            } else {
                "Barely a hit.".to_string()
            }
        },
        0 => {
            "A glancing blow.".to_string()
        },
        _ => {
            "A ??? attack".to_string()
        }
    }
}



