<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="spritesheet_tiles" tilewidth="128" tileheight="128" tilecount="128" columns="8">
 <image source="../kenney_platformer-pack-redux/Spritesheets/spritesheet_tiles.png" width="1024" height="2048"/>
 <tile id="25">
  <animation>
   <frame tileid="25" duration="100"/>
   <frame tileid="33" duration="100"/>
  </animation>
 </tile>
 <tile id="24">
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="0" y="0" width="128" height="128"/>
  </objectgroup>
 </tile>
 <tile id="32">
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="0" y="0" width="128" height="128"/>
  </objectgroup>
 </tile>
 <tile id="88">
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="13" y="20" width="102" height="102">
    <ellipse/>
   </object>
  </objectgroup>
 </tile>
 <tile id="104">
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="0" y="0">
    <polygon points="22,24 106,24 124,126 4,126"/>
   </object>
  </objectgroup>
 </tile>
</tileset>
