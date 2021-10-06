use rand::seq::SliceRandom;
use std::rc::Rc; 
use sixtyfps::Model;

fn main() {
    // MainWindow is a declaration + definition of a Window object (we instantiate one here) with
    // some properties like width (has a meaning to sixtyfps itself) and memory_tiles (user-defined) 
    let main_window = MainWindow::new();

    // for each prop, setters and getters are defined. the getter returns an Rc<dyn sixtyfps::Model>
    // I think in this case we get a VecModel (that implements aforementiond Model trait). we cant
    // modify a model in place, thats why we get a handle to it and collect an iter;
    let mut tiles_copy: Vec<TileData> = main_window.get_memory_tiles().iter().collect();
    
    // putting a second copy of the tiles to make pairs
    tiles_copy.extend(tiles_copy.clone());

    // shuffle tiles
    let mut rng = rand::thread_rng();
    tiles_copy.shuffle(&mut rng);

    // create a new tiles model, which is a vec model, from the vec. Remember model means impls Model.
    let tiles_model = Rc::new(sixtyfps::VecModel::from(tiles_copy));

    // we set our tiles_model to the one in place. also we clone it because we still wanna edit it for logic
    main_window.set_memory_tiles(sixtyfps::ModelHandle::new(tiles_model.clone()));

    // we're gonna move this ref into sth owned by main_window itself, so circular ref bad => weak ref 
    let main_window_weak = main_window.as_weak();
    main_window.on_check_if_open_pair_is_solved(move || {
        let mut flipped_tiles = 
            tiles_model.iter().enumerate().filter(|(_, tile)| tile.image_visible && !tile.solved);

        
        // let (tile1_index, tile1) = match flipped_tiles.next() {
        //     Some(t) => (t.0, t.1),
        // }

        // the tile in tuple can be destructured in place to have separate index and tile data, but me lazy
        if let (Some(mut t1), Some(mut t2)) = (flipped_tiles.next(), flipped_tiles.next()) {
            
            // t1.0 is t1 index and t1.1 is t1 data. if data match
            if t1.1 == t2.1 {
                t1.1.solved = true;
                t2.1.solved = true;
                tiles_model.set_row_data(t1.0, t1.1);
                tiles_model.set_row_data(t2.0, t2.1);

            } else {
                let main_window = main_window_weak.unwrap();
                main_window.set_disable_tiles(true);
                
                use sixtyfps::Timer;
                use std::time::Duration;

                let tiles_model = tiles_model.clone();
                Timer::single_shot(Duration::from_secs(1), move || {
                    main_window.set_disable_tiles(false);
                    t1.1.image_visible = false;
                    t2.1.image_visible = false;
                    tiles_model.set_row_data(t1.0, t1.1);
                    tiles_model.set_row_data(t2.0, t2.1);
                });
            }

        };

        
        if let (Some((t1_idx, mut t1)), Some((t2_idx, mut t2))) =
            (flipped_tiles.next(), flipped_tiles.next())
        {
            let is_pair_solved = t1 == t2;
            if is_pair_solved {
                t1.solved = true;
                tiles_model.set_row_data(t1_idx, t1);
                t2.solved = true;
                tiles_model.set_row_data(t2_idx, t2);
            } else {
                let main_window = main_window_weak.unwrap();
                main_window.set_disable_tiles(true);
                let tiles_model = tiles_model.clone();
                sixtyfps::Timer::single_shot(std::time::Duration::from_secs(1), move || {
                    main_window.set_disable_tiles(false);
                    t1.image_visible = false;
                    tiles_model.set_row_data(t1_idx, t1);
                    t2.image_visible = false;
                    tiles_model.set_row_data(t2_idx, t2);
                });
            }
        }
    });

    main_window.run();
}

sixtyfps::sixtyfps! {
    struct TileData := {
        image: image,
        image_visible: bool,
        solved: bool
    }

    MemoryTile := Rectangle {
        callback clicked;
        property <bool> solved: false;
        property <bool> open_curtain: false;
        property <image> icon;

        width: 64px;
        height: 64px;

        // this only applies to image and not to curtains. no border-radius-top-left in .60 :(
        border-radius: 8px;

        background: solved ? #34CE57 : #3960D5;
        animate background { duration: 800ms; }

        Image {
            source: icon;
            width: parent.width;
            height: parent.height;
        }

        // Left Curtain
        Rectangle {
            width: open_curtain ? 0px : parent.width / 2;
            animate width { duration: 250ms; easing: ease-in; }
            height: parent.height;
            background: #193076;
        }

        // Right Curtain
        Rectangle {
            x: open_curtain ? parent.width : parent.width / 2;
            animate x { duration: 250ms; easing: ease-in; }
            width: open_curtain ? 0px: parent.width / 2;
            animate width { duration: 250ms; easing: ease-in; }
            height: parent.height;
            background: #193076;
        }

        TouchArea {
            clicked => {
                // root is MemoryTile. we're delegating to user of MemoryTile (who supplies their clicked callback)
                root.clicked();
            }
        }
    }

    MainWindow := Window {
        width: 326px;
        height: 326px;

        callback check_if_open_pair_is_solved; // impled in rust. filters tiles for the 2 open and comps
        property <bool> disable_tiles; // while a 1 sec timeout is playing after a wrong guess 

        property <[TileData]> memory_tiles: [
            { image: @image-url("icons/at.png") },
            { image: @image-url("icons/balance-scale.png") },
            { image: @image-url("icons/bicycle.png") },
            { image: @image-url("icons/bus.png") },
            { image: @image-url("icons/cloud.png") },
            { image: @image-url("icons/cogs.png") },
            { image: @image-url("icons/motorcycle.png") },
            { image: @image-url("icons/video.png") },
        ];

        // create a MemoryTile for each TileData
        for tile[i] in memory_tiles: MemoryTile {
            x: mod(i, 4) * 74px; // i: {0th element, 4th element} => x is 0px (leftmost) 
            y: floor(i / 4) * 74px; // since i is a whole number and we floor, first row {0,0,0,0}
            width: 64px;
            height: 64px;
            icon: tile.image;
            open_curtain: tile.image_visible || tile.solved;
            // propagate solved status from TileData to MemoryTile
            solved: tile.solved;
            clicked => {
                if (!root.disable_tiles) {
                    tile.image_visible = !tile.image_visible;
                    root.check_if_open_pair_is_solved();
                }
            }
        }
    }
}