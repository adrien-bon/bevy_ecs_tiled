<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="drjamgo_hex_16x16" tilewidth="16" tileheight="16" tilecount="20" columns="4">
 <image source="tiles/drjamgo_hex_16x16.png" width="64" height="80"/>
 <tile id="0">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="user_properties::BiomeInfos">
    <properties>
     <property name="ty" propertytype="user_properties::BiomeType" value="Plain"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="1">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="user_properties::BiomeInfos">
    <properties>
     <property name="ty" propertytype="user_properties::BiomeType" value="Forest"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="2">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="user_properties::BiomeInfos">
    <properties>
     <property name="block_line_of_sight" type="bool" value="true"/>
     <property name="ty" propertytype="user_properties::BiomeType" value="Mountain"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="3">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="user_properties::BiomeInfos">
    <properties>
     <property name="ty" propertytype="user_properties::BiomeType" value="Water"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="5">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="user_properties::BiomeInfos">
    <properties>
     <property name="ty" propertytype="user_properties::BiomeType" value="Desert"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="6">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="user_properties::BiomeInfos">
    <properties>
     <property name="ty" propertytype="user_properties::BiomeType" value="Water"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="7">
  <properties>
   <property name="BiomeInfos" type="class" propertytype="user_properties::BiomeInfos">
    <properties>
     <property name="ty" propertytype="user_properties::BiomeType" value="Water"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="19">
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
