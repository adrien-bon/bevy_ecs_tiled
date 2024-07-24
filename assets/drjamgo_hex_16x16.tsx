<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="drjamgo_hex_16x16" tilewidth="16" tileheight="16" tilecount="20" columns="4">
 <image source="tiles/drjamgo_hex_16x16.png" width="64" height="80"/>
 <tile id="2" type="TileBundle">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="BiomeInfos">
    <properties>
     <property name="BlockLineOfSight" type="bool" value="true"/>
     <property name="Type" propertytype="BiomeType" value="Mountain"/>
    </properties>
   </property>
  </properties>
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
