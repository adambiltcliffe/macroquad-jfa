use macroquad::prelude::*;

const RENDER_W: f32 = 128.0;
const RENDER_H: f32 = 128.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform tile physics test".to_owned(),
        fullscreen: false,
        window_width: RENDER_W as i32,
        window_height: RENDER_H as i32,
        ..Default::default()
    }
}

fn get_screen_camera(width: f32, height: f32) -> Camera2D {
    Camera2D {
        zoom: (vec2(2. / width, 2. / height)),
        target: vec2(width / 2., height / 2.),
        ..Default::default()
    }
}

fn get_camera_for_target(target: &RenderTarget) -> Camera2D {
    let width = target.texture.width() as f32;
    let height = target.texture.height() as f32;
    Camera2D {
        render_target: Some(*target),
        zoom: (vec2(2. / width, 2. / height)),
        target: vec2(width / 2., height / 2.),
        ..Default::default()
    }
}

fn render_pass(src: &RenderTarget, dst: &RenderTarget, material: Material, color: Color) {
    set_camera(&get_camera_for_target(dst));
    gl_use_material(material);
    draw_texture_ex(
        src.texture,
        0.,
        0.,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(RENDER_W as f32, RENDER_H as f32)),
            ..Default::default()
        },
    );
}

fn encode_param(n: u32) -> Color {
    Color {
        r: (n as f32) / 256.0,
        ..Color::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let rt_geom = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_init = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_step1 = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_step2 = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_final = render_target(RENDER_W as u32, RENDER_H as u32);
    rt_geom.texture.set_filter(FilterMode::Nearest);
    rt_init.texture.set_filter(FilterMode::Nearest);
    rt_step1.texture.set_filter(FilterMode::Nearest);
    rt_step2.texture.set_filter(FilterMode::Nearest);
    rt_final.texture.set_filter(FilterMode::Nearest);
    let init_material = load_material(VERTEX_SHADER, FS_INIT, MaterialParams::default()).unwrap();
    let step_material = load_material(VERTEX_SHADER, FS_STEP, MaterialParams::default()).unwrap();
    let final_material = load_material(VERTEX_SHADER, FS_FINAL, MaterialParams::default()).unwrap();

    let mut n = 1;

    loop {
        set_camera(&get_camera_for_target(&rt_geom));
        gl_use_default_material();
        clear_background(BLACK);
        draw_rectangle(10.0, 40.0, 50.0, 20.0, WHITE);
        draw_rectangle(100.0, 75.0, 10.0, 60.0, WHITE);
        draw_rectangle(45.0, 75.0, 10.0, 60.0, WHITE);
        draw_rectangle(20.0, 100.0, 60.0, 10.0, WHITE);
        draw_triangle(vec2(2.0, 0.0), vec2(40.0, 0.0), vec2(40.0, 38.0), WHITE);
        draw_text_ex(
            &format!("{}: hello world", n),
            50.0,
            20.0,
            TextParams {
                font_size: 18,
                font_scale: -1.0, // macroquad renders text assuming that (0,0) is top left
                font_scale_aspect: -1.0,
                ..Default::default()
            },
        );

        render_pass(&rt_geom, &rt_init, init_material, WHITE);
        render_pass(&rt_init, &rt_step1, step_material, encode_param(32));
        render_pass(&rt_step1, &rt_step2, step_material, encode_param(16));
        render_pass(&rt_step2, &rt_step1, step_material, encode_param(8));
        render_pass(&rt_step1, &rt_step2, step_material, encode_param(4));
        render_pass(&rt_step2, &rt_step1, step_material, encode_param(2));
        render_pass(&rt_step1, &rt_step2, step_material, encode_param(1));
        render_pass(&rt_step2, &rt_final, final_material, encode_param(32));

        set_camera(&get_screen_camera(RENDER_W as f32, RENDER_H as f32));
        gl_use_default_material();
        draw_texture_ex(
            rt_final.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(RENDER_W as f32, RENDER_H as f32)),
                ..Default::default()
            },
        );

        next_frame().await;
        n += 1;
    }
}

const FS_INIT: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
    // if the red channel is over the intensity threshold then count it as a seed
    if (res.r > 0.3) {
        gl_FragColor = vec4((gl_FragCoord.x - 0.5) / 256.0, (gl_FragCoord.y - 0.5) / 256.0, 0.0, 1.0);
    } else {
        gl_FragColor = vec4(0.0);
    }
}
"#;

const FS_STEP: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    vec4 res = texture2D(Texture, uv);
    vec2 coords = gl_FragCoord.xy - vec2(0.5, 0.5);
    vec2 current_pos;
    float current_dist;
    if (res.a == 1.0) {
        current_pos = vec2(round(res.r * 256.0), round(res.g * 256.0));
        current_dist = length(coords - current_pos);
    } else {
        current_dist = 9999.9;
    }
    int r = int(color.r * 256.0);
    vec2 size = vec2(textureSize(Texture, 0));
    for (int dx = -1; dx <= 1; dx += 1) {
        for (int dy = -1; dy <= 1; dy += 1) {
            vec2 offs = vec2(float(dx * r), float(dy * r));
            vec2 newFragCoord = coords + offs;
            vec2 newuv = (newFragCoord + vec2(0.5, 0.5)) / size;
            vec4 other_res = texture2D(Texture, newuv);
            if (other_res.a == 1.0) {
                vec2 other_pos = vec2(round(other_res.r * 256.0), round(other_res.g * 256.0));
                float len = length(coords - other_pos);
                if (len < current_dist) {
                    current_dist = len;
                    current_pos = other_pos;
                }
            }
        }
    }
    gl_FragColor = vec4(current_pos.x / 256.0, current_pos.y / 256.0, current_dist, 1.0);
}
"#;

const FS_FINAL: &'static str = r#"#version 100
precision highp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    float r = color.r * 256.0;
    vec4 res = texture2D(Texture, uv);
    // if it was a seed, the alpha will be 1.0, so draw it as white
    if (res.a == 1.0) {
        vec2 current_pos = gl_FragCoord.xy - vec2(0.5, 0.5);
        vec2 encoded_pos = vec2(round(res.r * 256.0), round(res.g * 256.0));
        float len = length(current_pos - encoded_pos);
        if (len == 0.0) {
            gl_FragColor = vec4(1.0);
        } else if (len < r || true) {
            gl_FragColor = vec4(0.0, (r - len) / r, 0.0, 1.0);
        }
    } else {
        gl_FragColor = vec4(0.0);
    }
}
"#;

const VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";
