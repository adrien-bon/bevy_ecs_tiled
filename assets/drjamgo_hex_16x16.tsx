<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="drjamgo_hex_16x16" tilewidth="16" tileheight="16" tilecount="20" columns="4">
 <image source="tiles/drjamgo_hex_16x16.png" width="64" height="80"/>
 <tile id="2" type="TileBundle">
  <properties>
   <property name="BiomeInfos" type="class"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="3.15789" y="3.52941" width="9.47368" height="8.35913"/>
  </objectgroup>
 </tile>
 <tile id="10">
  <objectgroup draworder="index" id="2">
   <object id="1" x="5.387" y="4.27245" width="7.43034" height="7.80186">
    <ellipse/>
   </object>
  </objectgroup>
 </tile>
 <tile id="19" type="TileComponent">
  <properties>
   <property name="prefered_color" type="color" value="#ff09ec11"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="2" y="2.54545" width="12" height="10.5455"/>
  </objectgroup>
  <animation>
   <frame tileid="12" duration="200"/>
   <frame tileid="13" duration="200"/>
   <frame tileid="14" duration="200"/>
   <frame tileid="15" duration="200"/>
  </animation>
 </tile>
</tileset>
