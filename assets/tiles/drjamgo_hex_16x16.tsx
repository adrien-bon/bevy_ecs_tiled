<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.12.2" name="drjamgo_hex_16x16" tilewidth="16" tileheight="16" tilecount="20" columns="4">
 <image source="drjamgo_hex_16x16.png" width="64" height="80"/>
 <tile id="0">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="1"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Plain"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="1">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="2"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Plain"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
   <property name="Resource" type="class" propertytype="properties_basic::Resource">
    <properties>
     <property name=":variant" propertytype="properties_basic::Resource:::Variant" value="Wood"/>
    </properties>
   </property>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="2.18182" y="2.36364" width="12.7273" height="9"/>
  </objectgroup>
 </tile>
 <tile id="2">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="3"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Mountain"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="3" y="3.54545" width="10.2727" height="9.72727">
    <ellipse/>
   </object>
  </objectgroup>
 </tile>
 <tile id="3">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="99"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Water"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="4">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="1"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Forest"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
   <property name="Resource" type="class" propertytype="properties_basic::Resource">
    <properties>
     <property name=":variant" propertytype="properties_basic::Resource:::Variant" value="Strawberry"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="5">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="1"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Desert"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="6">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="99"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Water"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="7">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="99"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Water"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="8">
  <properties>
   <property name="Biome" type="class" propertytype="properties_basic::Biome">
    <properties>
     <property name="move_cost" type="int" value="1"/>
     <property name="types" type="class" propertytype="alloc::vec::Vec&lt;properties_basic::BiomeType&gt;">
      <properties>
       <property name="list" type="list">
        <item type="class" propertytype="properties_basic::BiomeType">
         <properties>
          <property name=":variant" propertytype="properties_basic::BiomeType:::Variant" value="Plain"/>
         </properties>
        </item>
       </property>
      </properties>
     </property>
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
