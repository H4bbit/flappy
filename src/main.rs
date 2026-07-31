use macroquad::prelude::*;

// -----------------------------------------------------------------------------
// Física
// -----------------------------------------------------------------------------

/// Aceleração vertical aplicada continuamente, em pixels por segundo ao quadrado.
const GRAVITY: f32 = 1400.0;

/// Velocidade vertical aplicada imediatamente quando o jogador bate as asas.
/// Em Macroquad, valores negativos movem o objeto para cima.
const FLAP_VELOCITY: f32 = -420.0;

/// Evita que o pássaro continue acelerando indefinidamente durante a queda.
const MAX_FALL_SPEED: f32 = 700.0;

// -----------------------------------------------------------------------------
// Rotação visual
// -----------------------------------------------------------------------------

/// Inclinação máxima enquanto o pássaro sobe.
const MAX_ROTATION_UP: f32 = -25.0;

/// Inclinação máxima enquanto o pássaro cai.
const MAX_ROTATION_DOWN: f32 = 75.0;

// -----------------------------------------------------------------------------
// Sprite
// -----------------------------------------------------------------------------

/// Região ocupada pelo pássaro dentro do atlas 512×512.
const BIRD_SOURCE_RECT: Rect = Rect {
    x: 3.0,
    y: 491.0,
    w: 17.0,
    h: 12.0,
};

/// Tamanho usado para desenhar o sprite na tela.
///
/// O sprite original possui apenas 17×12 pixels, então ele é ampliado
/// mantendo o filtro `Nearest` para preservar o pixel art.
const BIRD_DRAW_SIZE: Vec2 = Vec2::new(68.0, 48.0);

/// Exibe o atlas inteiro no canto superior esquerdo para conferir recortes.
const DEBUG_ATLAS: bool = false;

/// Exibe o centro lógico usado pela física e pelas colisões.
const DEBUG_BIRD_CENTER: bool = false;

// -----------------------------------------------------------------------------
// Bird
// -----------------------------------------------------------------------------

struct Bird {
    /// Centro lógico do pássaro em coordenadas da tela.
    position: Vec2,

    /// Velocidade vertical atual, em pixels por segundo.
    velocity_y: f32,
}

impl Bird {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity_y: 0.0,
        }
    }

    /// Substitui a velocidade vertical por um impulso fixo.
    ///
    /// Isso torna cada flap previsível, mesmo quando o pássaro já está
    /// subindo ou caindo.
    fn flap(&mut self) {
        self.velocity_y = FLAP_VELOCITY;
    }

    /// Atualiza velocidade, posição e limites verticais.
    fn update(&mut self, dt: f32, screen_height: f32) {
        self.apply_gravity(dt);
        self.move_vertically(dt);
        self.resolve_screen_bounds(screen_height);
    }

    fn apply_gravity(&mut self, dt: f32) {
        self.velocity_y += GRAVITY * dt;
        self.velocity_y = self.velocity_y.clamp(FLAP_VELOCITY, MAX_FALL_SPEED);
    }

    fn move_vertically(&mut self, dt: f32) {
        self.position.y += self.velocity_y * dt;
    }

    fn resolve_screen_bounds(&mut self, screen_height: f32) {
        let half_height = BIRD_DRAW_SIZE.y / 2.0;
        let ceiling = half_height;
        let floor = screen_height - half_height;

        if self.position.y < ceiling {
            self.position.y = ceiling;
            self.velocity_y = 0.0;
        } else if self.position.y > floor {
            self.position.y = floor;
            self.velocity_y = 0.0;
        }
    }

    /// Converte a velocidade vertical atual em uma inclinação visual.
    fn rotation(&self) -> f32 {
        remap(
            self.velocity_y,
            FLAP_VELOCITY,
            MAX_FALL_SPEED,
            MAX_ROTATION_UP,
            MAX_ROTATION_DOWN,
        )
        .clamp(MAX_ROTATION_UP, MAX_ROTATION_DOWN)
        .to_radians()
    }

    fn draw(&self, atlas: &Texture2D) {
        // `draw_texture_ex` recebe o canto superior esquerdo, enquanto a física
        // trabalha com o centro do pássaro. Por isso fazemos esta conversão.
        let draw_position = self.position - BIRD_DRAW_SIZE / 2.0;

        draw_texture_ex(
            atlas,
            draw_position.x,
            draw_position.y,
            WHITE,
            DrawTextureParams {
                source: Some(BIRD_SOURCE_RECT),
                dest_size: Some(BIRD_DRAW_SIZE),
                rotation: self.rotation(),

                // Com `source` e `dest_size`, o pivô explícito causava um
                // deslocamento inesperado. O pivô padrão mantém o sprite
                // acompanhando corretamente sua posição lógica.
                pivot: None,

                ..Default::default()
            },
        );

        if DEBUG_BIRD_CENTER {
            draw_circle(self.position.x, self.position.y, 2.0, RED);
        }
    }
}

// -----------------------------------------------------------------------------
// Utilitários
// -----------------------------------------------------------------------------

/// Converte linearmente um valor de um intervalo para outro.
///
/// Aqui, a velocidade vertical é transformada em um ângulo de rotação.
fn remap(value: f32, input_min: f32, input_max: f32, output_min: f32, output_max: f32) -> f32 {
    let input_range = input_max - input_min;
    let output_range = output_max - output_min;

    output_min + (value - input_min) * output_range / input_range
}

fn flap_pressed() -> bool {
    is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space)
}

// -----------------------------------------------------------------------------
// Game loop
// -----------------------------------------------------------------------------

#[macroquad::main("Flappy")]
async fn main() -> Result<(), macroquad::Error> {
    let atlas = load_texture("assets/sprite.png").await?;
    atlas.set_filter(FilterMode::Nearest);

    println!("Atlas carregado: {}x{}", atlas.width(), atlas.height());

    let initial_position = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let mut bird = Bird::new(initial_position);

    loop {
        let dt = get_frame_time();

        if flap_pressed() {
            bird.flap();
        }

        bird.update(dt, screen_height());

        clear_background(SKYBLUE);

        if DEBUG_ATLAS {
            draw_texture(&atlas, 0.0, 0.0, WHITE);
        }

        bird.draw(&atlas);

        next_frame().await;
    }
}
