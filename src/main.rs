use macroquad::{input, prelude::*};

const PADDLE_WIDTH: f32 = 128.0;
const PADDLE_HEIGHT: f32 = 18.0;

const BALL_RADIUS: f32 = 18.0;
const BALL_SPEED: f32 = 10.0;

const QUAD_WIDTH: f32 = 96.0;
const QUAD_HEIGHT: f32 = 28.0;

enum InputAction {
    None,
    MoveLeft,
    MoveRight,
    PosX(f32),
}

struct Paddle {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: Color,
}

impl Paddle {
    fn new() -> Self {
        Self {
            x: screen_width() / 2.0 - PADDLE_WIDTH / 2.0,
            y: screen_height() - 64.0 - PADDLE_HEIGHT / 2.0,
            w: PADDLE_WIDTH,
            h: PADDLE_HEIGHT,
            color: BLUE,
        }
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        }
    }

    fn global_center_x(&self) -> f32 {
        self.x + self.center_x()
    }

    fn global_center_y(&self) -> f32 {
        self.y + self.center_y()
    }

    fn center_x(&self) -> f32 {
        self.w / 2.0
    }

    fn center_y(&self) -> f32 {
        self.h / 2.0
    }

    fn update(&mut self, input_action: &InputAction) {
        use InputAction::*;

        match input_action {
            MoveLeft => self.x -= 10.0,
            MoveRight => self.x += 10.0,
            PosX(x) => self.x = *x - self.center_x(),
            _ => {}
        }

        self.x = self.x.clamp(0.0, screen_width() - self.w);
    }

    fn reset(&mut self) {
        self.x = screen_width() / 2.0 - PADDLE_WIDTH / 2.0;
        self.y = screen_height() - 64.0 - PADDLE_HEIGHT / 2.0;
        self.w = PADDLE_WIDTH;
        self.h = PADDLE_HEIGHT;
    }

    fn render(&self) {
        draw_rectangle(self.x, self.y, self.w, self.h, self.color);
    }
}

struct Ball {
    x: f32,
    y: f32,
    r: f32,
    color: Color,
    vel: Vec2,
}

impl Ball {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y: y - BALL_RADIUS,
            r: BALL_RADIUS,
            color: RED,
            vel: vec2(1.0, -1.0),
        }
    }

    fn circle(&self) -> Circle {
        Circle {
            x: self.x,
            y: self.y,
            r: self.r,
        }
    }

    fn set_position(&mut self, pos: (f32, f32)) {
        self.x = pos.0;
        self.y = pos.1 - BALL_RADIUS;
    }

    fn update(&mut self) {
        self.x += self.vel.x * BALL_SPEED;
        self.y += self.vel.y * BALL_SPEED;
    }

    fn reset(&mut self) {
        self.vel = vec2(1.0, -1.0);
    }

    fn render(&self) {
        draw_circle(self.x, self.y, self.r, self.color);
    }
}

#[derive(Debug)]
struct Quad {
    rect: Rect,
    color: Color,
}

impl Quad {
    fn new(rect: Rect, color: Color) -> Self {
        Self { rect, color }
    }
}

fn circle_rectangle_collision(c: &Circle, r: &Rect) -> bool {
    // Find the vertical & horizontal (distX/distY) distances between the circle’s center and the rectangle’s center
    let dist_x = ((c.x + c.r) - (r.x + r.w / 2.0)).abs();
    let dist_y = ((c.y + c.r) - (r.y + r.h / 2.0)).abs();

    // If the distance is greater than halfCircle + halfRect, then they are too far apart to be colliding
    if r.w / 2.0 + c.r < dist_x {
        return false;
    }
    if r.h / 2.0 + c.r < dist_y {
        return false;
    }

    // If the distance is less than halfRect then they are definitely colliding
    if dist_x <= r.w / 2.0 {
        return true;
    }
    if dist_y <= r.h / 2.0 {
        return true;
    }

    // Test for collision at rect corner.
    // Think of a line from the rect center to any rect corner
    // Now extend that line by the radius of the circle
    // If the circle’s center is on that line they are colliding at exactly that rect corner
    // Using Pythagoras formula to compare the distance between circle and rect centers.

    let dx = dist_x - r.w / 2.0;
    let dy = dist_y - r.h / 2.0;

    dx * dx + dy * dy <= c.r * c.r
}

fn generate_quads() -> Vec<Quad> {
    let columns = (screen_width() / QUAD_WIDTH) as i32 - 1;
    let mut quads: Vec<Quad> = vec![];
    let left_margin = (screen_width() - (QUAD_WIDTH * columns as f32)) / 2.0;

    // Quads
    let w = QUAD_WIDTH - 4.0;
    let h = QUAD_HEIGHT;
    let y = 32.0;

    for i in 0..columns {
        let x = left_margin + (QUAD_WIDTH * i as f32);
        quads.push(Quad::new(Rect { x, y, w, h }, GREEN));
    }
    quads
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Break Quad".to_string(),
        window_width: 640,
        window_height: 480,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut quads: Vec<Quad> = generate_quads();

    let mut prev_mouse_pos = mouse_position();

    let mut paddle = Paddle::new();
    let mut ball = Ball::new(paddle.global_center_x(), paddle.y);

    let mut is_launched = false;
    let mut score = 0;

    loop {
        // Logic
        if input::is_key_down(KeyCode::Escape) {
            break;
        }

        let mut input_action = InputAction::None;
        if input::is_key_down(KeyCode::A) || input::is_key_down(KeyCode::Left) {
            input_action = InputAction::MoveLeft;
        } else if input::is_key_down(KeyCode::D) || input::is_key_down(KeyCode::Right) {
            input_action = InputAction::MoveRight;
        } else if !is_launched
            && (input::is_key_down(KeyCode::Space)
                || input::is_mouse_button_released(MouseButton::Left))
        {
            is_launched = true;
        } else if input::is_key_released(KeyCode::R) {
            is_launched = false;
            paddle.reset();
            ball.reset();
            quads = generate_quads();
        }

        if input::mouse_position() != prev_mouse_pos {
            input_action = InputAction::PosX(mouse_position().0);
            prev_mouse_pos = mouse_position();
        }

        // Update
        paddle.update(&input_action);

        if is_launched {
            ball.update();
        } else {
            ball.set_position((paddle.global_center_x(), paddle.y));
        }

        // FIXME: I AM BUG
        // if paddle.y < ball.y + ball.r && paddle.x < ball.x && ball.x + ball.r < paddle.x + paddle.w
        // {
        //     ball.vel.y = -ball.vel.y;
        // } else
        if ball.x <= 0.0 || screen_width() <= ball.x + ball.r {
            ball.vel.x *= -1.0;
        } else if ball.y + ball.r <= 0.0 {
            ball.vel.y *= -1.0;
        } else if screen_height() <= ball.y {
            is_launched = false;
            paddle.reset();
            ball.reset();
        }

        if circle_rectangle_collision(&ball.circle(), &paddle.rect()) {
            ball.vel.y *= -1.0;
        }

        quads = quads
            .into_iter()
            .filter(|q| {
                if circle_rectangle_collision(&ball.circle(), &q.rect) {
                    score += 100;
                    false
                } else {
                    true
                }
            })
            .collect();

        // Render
        clear_background(BLACK);

        // Paddle
        paddle.render();
        ball.render();

        for q in &quads {
            draw_rectangle(q.rect.x, q.rect.y, q.rect.w, q.rect.h, q.color);
        }

        let s = format!("Score {}", score);
        draw_text(s.as_str(), 24.0, 24.0, 32.0, WHITE);

        next_frame().await
    }
}
