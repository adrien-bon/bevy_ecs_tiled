<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="orthogonal_1" tilewidth="32" tileheight="32" tilecount="7" columns="0">
 <grid orientation="orthogonal" width="1" height="1"/>
 <tile id="0" type="TileComponent">
  <image source="orthogonal/tile0.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collider" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="1" type="TileComponent">
  <properties>
   <property name="RANDOM PROPERTY THAT WILL NOT BE LOADED" type="bool" value="true"/>
  </properties>
  <image source="orthogonal/tile1.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="4" name="collision" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="2" type="TileComponent">
  <image source="orthogonal/tile2.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="3" name="collision" x="15.0398" y="1.81818">
    <polygon points="0,0 14.7784,5.80779 10.8665,15.6364 -13.0398,10.4242"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="3" type="TileComponent">
  <image source="orthogonal/tile3.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="4" type="TileComponent">
  <image source="orthogonal/tile4.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="2" name="collision" x="10.4301" y="21.0898" width="20.7273" height="10.3636">
    <ellipse/>
   </object>
  </objectgroup>
 </tile>
 <tile id="5" type="TileComponent">
  <image source="orthogonal/tile5.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="6" type="TileComponent">
  <image source="orthogonal/tile6.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="6.83272" y="1.23615" width="30.1959" height="7.93289" rotation="54.8082"/>
  </objectgroup>
 </tile>
</tileset>
