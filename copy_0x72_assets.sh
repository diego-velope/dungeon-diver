#!/usr/bin/env bash
# Run this from your dungeon-diver project root.
# It copies only the files your game actually uses from 0x72 DungeonTileset II v1.7.
# Adjust SRC to wherever you extracted the zip.

SRC="$HOME/Downloads/0x72_DungeonTilesetII_v1.7"
DEST="assets/0x72"

if [ ! -d "$SRC" ]; then
  echo "ERROR: Source not found at $SRC"
  echo "Edit the SRC variable at the top of this script."
  exit 1
fi

mkdir -p "$DEST"

echo "Copying terrain tiles..."
for f in \
  floor_1 floor_2 floor_3 floor_4 floor_5 floor_6 floor_7 floor_8 \
  wall_mid wall_top_mid wall_top_left wall_top_right \
  wall_left wall_right
do
  cp "$SRC/frames/${f}.png" "$DEST/"
  echo "  ✓ ${f}.png"
done

echo "Copying door..."
cp "$SRC/frames/doors_leaf_closed.png" "$DEST/"
cp "$SRC/frames/doors_leaf_open.png"   "$DEST/"
cp "$SRC/frames/doors_frame_top.png"   "$DEST/"
echo "  ✓ doors (3 files)"

echo "Copying UI..."
cp "$SRC/frames/ui_heart_full.png"  "$DEST/"
cp "$SRC/frames/ui_heart_empty.png" "$DEST/"
cp "$SRC/frames/ui_heart_half.png"  "$DEST/"
echo "  ✓ hearts (3 files)"

echo "Copying items..."
for f in \
  coin_anim_f0 coin_anim_f1 coin_anim_f2 coin_anim_f3 \
  flask_red flask_blue flask_big_red flask_big_blue \
  chest_full_open_anim_f0 chest_full_open_anim_f1 chest_full_open_anim_f2 \
  chest_empty_open_anim_f0 chest_empty_open_anim_f1 chest_empty_open_anim_f2 \
  skull crate
do
  cp "$SRC/frames/${f}.png" "$DEST/"
  echo "  ✓ ${f}.png"
done

echo ""
echo "Copying enemies (for Phase 4)..."
for f in \
  goblin_idle_anim_f0 goblin_idle_anim_f1 goblin_idle_anim_f2 goblin_idle_anim_f3 \
  goblin_run_anim_f0  goblin_run_anim_f1  goblin_run_anim_f2  goblin_run_anim_f3  \
  skelet_idle_anim_f0 skelet_idle_anim_f1 skelet_idle_anim_f2 skelet_idle_anim_f3 \
  skelet_run_anim_f0  skelet_run_anim_f1  skelet_run_anim_f2  skelet_run_anim_f3  \
  tiny_zombie_idle_anim_f0 tiny_zombie_idle_anim_f1 \
  tiny_zombie_idle_anim_f2 tiny_zombie_idle_anim_f3 \
  tiny_zombie_run_anim_f0  tiny_zombie_run_anim_f1  \
  tiny_zombie_run_anim_f2  tiny_zombie_run_anim_f3
do
  cp "$SRC/frames/${f}.png" "$DEST/"
  echo "  ✓ ${f}.png"
done

echo ""
echo "Done! $(ls $DEST | wc -l) files in $DEST"
