

use imageproc::image::{ImageReader, RgbImage, Rgb, math::Rect};

use imageproc::compose::{ crop, replace_mut };
use imageproc::map::map_pixels;

use bracket_lib::prelude::to_cp437;
use bracket_lib::terminal::{RGBA, Point};


use n_star::map::{Map, Tile};
use n_star::map::tile_render::{FixedTileRender, Wall4WayTileRender, TileRenderContext, TileDrawType};

use n_star::mapgen::MapGenerator;



const EIGHT_WAYS: [(i32, i32); 8] = [(-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1)];


fn render_map_image(tiles: &RgbImage, map: &Map ) -> RgbImage {
    // draw map
    let mut out_img = RgbImage::from_pixel( map.tiles.dim().0 as u32 * 12, map.tiles.dim().1 as u32 * 12, Rgb([0,0,0]) );


    for x in 0..(map.tiles.dim().0 as i32) {
        for y in 0..(map.tiles.dim().1 as i32) {

            let tidx = map.tiles[[ x as usize, y as usize ]];
            let tile = &map.tileset[tidx];

            let mut neighbors = [None; 8];

            for (idx, offs) in EIGHT_WAYS.iter().enumerate() {
                let ox = x + offs.0;
                let oy = y + offs.1;

                if ox < 0 || ox >= map.tiles.dim().0 as i32 || oy < 0 || oy >= map.tiles.dim().1 as i32 {
                    continue;
                }

                neighbors[idx] = Some( map.tiles[[ox as usize, oy as usize]] );
            }

            let tr_ctx = TileRenderContext{
                me: tidx,
                neighbors
            };

            let mut fg_color : RGBA = (0,0,0).into();
            let mut bg_color : RGBA = (0,0,0).into();
            let mut character : char = ' ';

            match tile.tr.get_draw(tr_ctx, &map.tileset) {
                TileDrawType::Dont => {continue;}
                TileDrawType::Regular(ch, flip) => {
                    character = ch;
                    if flip {
                        fg_color = tile.bg.into();
                        bg_color = tile.fg.into();
                    } else {
                        fg_color = tile.fg.into();
                        bg_color = tile.bg.into();
                    }
                },
                TileDrawType::Override{ch, fg, bg} => {
                    character = ch;
                    fg_color = fg.into();
                    bg_color = bg.into();
                }
            }

            let idx = to_cp437(character) as u32;

            let mut i = crop( &tiles, Rect{ x: (idx % 16) * 12, y: (idx / 16) * 12, width: 12, height: 12 } );

            i = map_pixels( &i, |p| { if p.0[0] > 128 { Rgb( [(fg_color.r * 256.0) as u8, (fg_color.g * 256.0) as u8, (fg_color.b * 256.0) as u8] ) } else { Rgb( [(bg_color.r * 256.0) as u8, (bg_color.g * 256.0) as u8, (bg_color.b * 256.0) as u8] ) } } );

            replace_mut( &mut out_img, &i, x as u32 * 12, y as u32 * 12 );
        }
    }


    out_img
}

fn main() {

    let font = ImageReader::open("res/cp437_12x12.png").unwrap().decode().unwrap();
    let font = font.into_rgb8();
    let mg = MapGenerator{w: 128, h: 128};

    for n in 0..8 {

        let bt = Tile{
            fg: (96, 96, 96),
            bg: (0,0,0),
            tr: Box::new( FixedTileRender{ch: '.'} ),
            passable: true,
            opaque: false
        };
        let wt = Tile{
            fg: (64, 128, 208),
            bg: (0,0,0),
            tr:  Box::new( Wall4WayTileRender{
                connects: vec![1usize].drain(..).collect(),
                           lower:('▄', false),
                           upper:('▀', false),
                           left:('▌', false),
                           right:('▐', false),
                           horizontal:('─', true),
                           vertical:('│', true),
                           misc:('■', false)
            } ),
            passable: false,
            opaque: true
        };
        let gt = Tile{
            fg: (89, 86, 85),
            bg: (0,0,0),
            tr:  Box::new( Wall4WayTileRender{
                connects: vec![1usize, 2usize].drain(..).collect(),
                           lower:('▄', false),
                           upper:('▀', false),
                           left:('▌', false),
                           right:('▐', false),
                           horizontal:('▓', false),
                           vertical:('▓', false),
                           misc:(' ', true)
            } ),
            passable: false,
            opaque: true
        };
        let gf = Tile{
            fg: (133, 131, 130),
            bg: (0,0,0),
            tr: Box::new( FixedTileRender{ch: '*'} ),
            passable: true,
            opaque: false
        };

        let map = mg.generate_map(
            vec![ bt, wt, gt, gf ]
        );

        let map_img = render_map_image( &font, &map );

        map_img.save( format!("maptest/out_{}.png", n) );

    }
}

