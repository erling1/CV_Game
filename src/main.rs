use macroquad::prelude::*;
use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::audio::{load_sound, play_sound, play_sound_once, PlaySoundParams};
use crate::miniquad::window;
use macroquad::ui::{hash, root_ui, Skin};

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";


struct Shape {
    size: f32,
    speed: f32,
    x: f32, 
    y: f32, 
    collided: bool,
}

impl Shape {
    fn collides_with(&self, other: &Self) -> bool {
        let distance_between_centers: f32 = ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt();
        let square_diagonal: f32 = (other.rect().w.powi(2) + other.rect().h.powi(2)).sqrt();

        if distance_between_centers <= self.size / 2.0 + square_diagonal / 2.0 {
            return true;
        }

        self.rect().overlaps(&other.rect())
    }
    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
      }
    }
}


enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
    GameWon,
}





#[macroquad::main("CV Game ")]
async fn main() {
    
    //loading in textures to use in game 
    set_pc_assets_folder("assets");
    let coffee_cup_texture: Texture2D = load_texture("coffee_cup.png").await.expect("Couldn't load coffe cup png file");
    coffee_cup_texture.set_filter(FilterMode::Nearest);
    let tired_intern_texture: Texture2D = load_texture("tired_intern.png").await.expect("Couldn't load tired intern  file");
    tired_intern_texture.set_filter(FilterMode::Nearest);
    let cv_mcguffin_texture: Texture2D = load_texture("cv_mcguffin.png").await.expect("Couldnt load cv_mcguffin file");

    build_textures_atlas();


    //load images to use for menues and define button and window style  
    let window_background = load_image("main_menu_background_creative_commons.png").await.expect("Couldnt load main menu background file");
    let button_background = load_image("button_background.png").await.expect("Couldnt load button background file");
    let button_clicked_background = load_image("button_clicked_background.png").await.expect("Couldnt load button clicked background file");
    let button_hovered_background = load_image("button_hovered_background.png").await.expect("Couldnt load button hovered background file");

    let see_cv_button_background = load_image("button_hovered_background.png").await.expect("Couldnt load button hovered background file");
    let gamewon_menu_background = load_image("button_hovered_background.png").await.expect("Couldnt load button hovered background file");



    let font = load_file("atari_games.ttf").await.expect("Couldnt load font file");

    let window_style = root_ui()
        .style_builder()
        .background(window_background)
        .background_margin(RectOffset::new(32.0, 76.0, 44.0, 20.0))
        .margin(RectOffset::new(0.0, -40.0, 0.0, 0.0)) 
        .build();

    let button_style = root_ui()
        .style_builder()
        .background(button_background)
        .background_hovered(button_hovered_background)
        .background_clicked(button_clicked_background)
        .background_margin(RectOffset::new(16.0, 16.0, 16.0, 16.0))
        .margin(RectOffset::new(16.0, 0.0, -8.0, -8.0))
        .font(&font)
        .expect("Failed creating button style")
        .text_color(WHITE)
        .font_size(64)
        .build();
    
    let see_cv_button_style = root_ui()
        .style_builder()
        .background(button_background)
        .background_hovered(button_hovered_background)
        .background_clicked(button_clicked_background)
        .background_margin(RectOffset::new(16.0, 16.0, 16.0, 16.0))
        .margin(RectOffset::new(16.0, 0.0, -8.0, -8.0))
        .font(&font)
        .expect("Failed creating button style")
        .text_color(WHITE)
        .font_size(64)
        .build();
    



    let label_style = root_ui()
        .style_builder()
        .font(&font)
        .expect("Failed creating label style")
        .text_color(WHITE)
        .font_size(28)
        .build();

    let ui_skin = Skin {
        window_style,
        button_style,
        label_style,
        ..root_ui().default_skin()
    };

    let gamewon_skin = Skin {
        window_style,
        see_cv_button_style,
        label_style,
        ..root_ui().default_skin()
    };

    root_ui().push_skin(&ui_skin);
    let window_size = vec2(screen_width() * 0.7, screen_height() * 0.7);

    //loading soud effects, these would be nice if they could be custom
    let theme_music = load_sound("8bit-spaceshooter.ogg").await.expect("Could not load theme music file");
    let sound_explosion = load_sound("explosion.wav").await.expect("Could not load explosion sound file");
    let sound_laser = load_sound("laser.wav").await.expect("Could not load laser sound file");

    play_sound(
        &theme_music,
        PlaySoundParams {
            looped: true,
            volume: 1.,
        },
    );



    //Set up shader, shader code is out of scope 
    let mut direction_modifier: f32 = 0.0;
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("iResolution", UniformType::Float2),
                UniformDesc::new("direction_modifier", UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .unwrap();
    
    //Defining movement speed based on monitor framerate, mine is quite high 
    const MOVEMENT_SPEED: f32 = 10.0;
    
    //seed 
    rand::srand(miniquad::date::now() as u64);
    
    //empty vec for squares (enemy ships)
    let mut squares = vec![];

    //empty vec for Enemu Bullets
    //todo 

    //empty vec for hero bullets
    let mut bullets: Vec<Shape> = vec![];

    //Defining hero shape 
    let mut circle = Shape {
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_width()/2.0, 
        y: screen_height()/2.0,
        collided: false,
    };

    //Defining CV collectables 
    let mut cv_mcguffin: Vec<Shape> = vec![];
    let mut mcguffin_score: i32 = 0;

    let mut game_state = GameState::MainMenu; 

    loop {
        clear_background(BLACK);

        material.set_uniform("iResolution", (screen_width(), screen_height()));
        material.set_uniform("direction_modifier", direction_modifier);
        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();



        match game_state {
            GameState::MainMenu => {

                //let button_width = window_size * 0.3;
                //let button_height = button_width * 0.25;
                let window_size_menu = vec2(screen_width() * 0.7, screen_height() * 0.7);
                let window_width = window_size_menu.x;
                let window_height = window_size_menu.y;
                
                let deploy_button_pos = vec2(window_width * 0.1, window_height / 2.0 - 40.0);
                let quit_button_pos = vec2(window_width * 0.1, window_height / 2.0 + 40.0);


                root_ui().window(
                    hash!(),
                    vec2(
                        screen_width() / 2.0 - window_size.x / 2.0,
                        screen_height() / 2.0 - window_size.y / 2.0,
                    ),
                    window_size_menu,
                    |ui| {
                        ui.label(vec2(80.0, -34.0), "Main Menu");
                        if ui.button(deploy_button_pos, "DEPLOY") {
                            squares.clear();
                            bullets.clear();
                            //todo explosions.clear();
                            circle.x = screen_width() / 2.0;
                            circle.y = screen_height() / 2.0;
                            mcguffin_score = 0; // dont need this yet, but it would be nice to implmenet a history of scorse and so on 
                            game_state = GameState::Playing;
                        }
                        if ui.button(quit_button_pos, "Send me back to my cubicle") {
                            std::process::exit(0);
                        }
                    },
                );
            },

            GameState::Playing => {


                let delta_time = get_frame_time();

                //*********** Circle Movement ****************
                if is_key_down(KeyCode::Right) {
                    circle.x += MOVEMENT_SPEED + delta_time; 
                    //also move stars when circle is moved around 
                    direction_modifier += 0.05 * delta_time;
                }
                if is_key_down(KeyCode::Left) {
                    circle.x -= MOVEMENT_SPEED + delta_time; 
                    direction_modifier -= 0.05 * delta_time;

                }
                if is_key_down(KeyCode::Down) {
                    circle.y += MOVEMENT_SPEED + delta_time; 
                }
                if is_key_down(KeyCode::Up) {
                    circle.y -= MOVEMENT_SPEED + delta_time; 
                }   


                circle.x = clamp(circle.x, 0.0, screen_width());
                circle.y = clamp(circle.y, 0.0, screen_height());
                
                //*********** Bullets Movement and Generation ****************
                if is_key_pressed(KeyCode::Space) {
                    bullets.push(Shape {
                        x: circle.x,
                        y: circle.y,
                        speed:circle.speed * 20.0,
                        size: circle.size * 0.3,
                        collided:false,
                    });

                    play_sound_once(&sound_laser)

                }

                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta_time; 
                }
                
                //*********** CV Mcgufin Movement and Generation ****************
                if rand::gen_range(0,200) >= 199 && cv_mcguffin.len()<= 5 {
                    let size = rand::gen_range(20.0, 40.0);
                    cv_mcguffin.push(Shape {
                        size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size/2.0, screen_width() - size/2.0),
                        y: -size,
                        collided:false,
                    })
                }
 
                        
                //*********** Square Movement and Generation ****************
                if rand::gen_range(0,99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    squares.push(Shape {
                        size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size/2.0, screen_width() - size/2.0),
                        y: -size,
                        collided:false,
                    });
                }

                for square in &mut squares {
                    square.y += square.speed * delta_time;
                }
                
                for mcguffin in &mut cv_mcguffin {
                    mcguffin.y +=mcguffin.speed * delta_time;
                }


                if squares.iter().any(|square| circle.collides_with(&square)) {
                    game_state = GameState::GameOver;
                }

                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }

                if mcguffin_score == 5 {
                    game_state = GameState::GameWon;
                }
                

                for square in &mut squares {
                    for bullet in &mut bullets {
                        if bullet.collides_with(&square){
                            square.collided = true;
                            bullet.collided = true; 
                        }
                    }
                }
                
                for mcguffin in &mut cv_mcguffin {
                    for bullet in &mut bullets {
                        if bullet.collides_with(&mcguffin){
                            mcguffin.collided = true;
                            bullet.collided = true; 
                            mcguffin_score += 1;
                        }
                    }
                }

                
                //remove when outside of screen 
                squares.retain(|square| square.y < screen_height() + square.size);
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);        
                cv_mcguffin.retain(|mcguffin| mcguffin.y < screen_height() + mcguffin.size);

                // remove when collided
                squares.retain(|square| !square.collided);
                bullets.retain(|bullet| !bullet.collided);
                cv_mcguffin.retain(|mcguffin| !mcguffin.collided);
                //Fancier Rectables :  draw_rectangle_ex()
            

                //*********** Draw everthing ****************        
                //draw_circle(circle.x, circle.y, circle.size/2.0, YELLOW);
                draw_texture_ex(
                    &coffee_cup_texture,
                    circle.x, 
                    circle.y, 
                    //circle.size/2.0, 
                    YELLOW,
                    DrawTextureParams {
                        dest_size: Some(vec2(circle.size, circle.size)),
                        ..Default::default()
                        },
                    );
                for square in &squares {
                    draw_texture_ex(
                        &tired_intern_texture,
                        square.x - square.size / 2.0,
                        square.y - square.size / 2.0,
                     //   square.size,
                     //   square.size,
                        PINK,
                        DrawTextureParams {
                            dest_size: Some(vec2(square.size, square.size)),
                            ..Default::default()
                        },
                    );        
                }

                for mcguffin in &cv_mcguffin {
                    draw_texture_ex(
                        &cv_mcguffin_texture,
                        mcguffin.x - mcguffin.size / 2.0,
                        mcguffin.y - mcguffin.size / 2.0,
                        GREEN,
                        DrawTextureParams {
                            dest_size: Some(vec2(mcguffin.size, mcguffin.size)),
                            ..Default::default()
                        },
                    );        
                }


                for bullet in &bullets {
                    draw_rectangle(
                        bullet.x - bullet.size / 2.0,
                        bullet.y - bullet.size / 2.0,
                        bullet.size,
                        bullet.size,
                        PINK,
                    );        
                }

            },

            GameState::Paused => {

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Playing;
                }
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );

            },

            GameState::GameOver => {

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::MainMenu;
                }
                let text = "GAME OVER!";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    RED,
                );

            },

            GameState::GameWon => {




                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::MainMenu
                }

                let text = "Well played!\nHere’s my CV,\nofficially the real prize.";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    GREEN,
                );

                    
                let window_size_menu = vec2(screen_width() * 0.7, screen_height() * 0.7);
                let window_width = window_size_menu.x;
                let window_height = window_size_menu.y;
                
                let deploy_button_pos = vec2(window_width * 0.1, window_height / 2.0 - 40.0);
                let quit_button_pos = vec2(window_width * 0.1, window_height / 2.0 + 40.0);



                root_ui().window(
                    hash!(),
                    vec2(
                        screen_width() / 2.0 - window_size.x / 2.0,
                        screen_height() / 2.0 - window_size.y / 2.0,
                    ),
                    window_size_menu,
                    |ui| {
                        ui.label(vec2(80.0, -34.0), "Game Won");
                        if ui.button(see_CV_button_pos, "See CV") {
                            squares.clear();
                            bullets.clear();
                            //todo explosions.clear();
                            circle.x = screen_width() / 2.0;
                            circle.y = screen_height() / 2.0;
                            mcguffin_score = 0; // dont need this yet, but it would be nice to implmenet a history of scorse and so on 
                         // let _ = win.open_with_url("https://.html");

                            //std::process::exit(0); not sure i                              
                        }
                    },
                );
               
                //todo: integrate a simple cv in my kubernetes api 
                //let _ = win.open_with_url("https://.html");
            }
        
        }


        next_frame().await
    }
}
