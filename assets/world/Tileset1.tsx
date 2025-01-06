<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="Tileset1" tilewidth="32" tileheight="32" tilecount="7" columns="0">
 <grid orientation="orthogonal" width="1" height="1"/>
 <tile id="0" type="TileComponent">
  <properties>
   <property name="spawn" type="class" propertytype="user_properties_avian::SpawnBundle">
    <properties>
     <property name="collider" type="class" propertytype="user_properties_avian::ColliderComponent">
      <properties>
       <property name="damage_per_second" type="float" value="12"/>
      </properties>
     </property>
    </properties>
   </property>
   <property name="test" type="class" propertytype="avian2d::position::Position"/>
  </properties>
  <image source="tiles/tile0.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collider" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="1" type="TileComponent">
  <properties>
   <property name="prefered_color" type="color" value="#ff0000de"/>
   <property name="spawn" type="class" propertytype="user_properties_avian::SpawnBundle">
    <properties>
     <property name="collider" type="class" propertytype="user_properties_avian::ColliderComponent">
      <properties>
       <property name="damage_per_second" type="float" value="12"/>
      </properties>
     </property>
    </properties>
   </property>
   <property name="test" type="class" propertytype="avian2d::position::Position"/>
  </properties>
  <image source="tiles/tile1.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="4" name="collision" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="2" type="TileComponent">
  <properties>
   <property name="prefered_color" type="color" value="#ff0000de"/>
   <property name="spawn" type="class" propertytype="user_properties_avian::SpawnBundle">
    <properties>
     <property name="collider" type="class" propertytype="user_properties_avian::ColliderComponent">
      <properties>
       <property name="damage_per_second" type="float" value="12"/>
      </properties>
     </property>
    </properties>
   </property>
   <property name="test" type="class" propertytype="avian2d::position::Position">
    <properties>
     <property name="0" type="class" propertytype="glam::Vec2">
      <properties>
       <property name="x" type="float" value="58"/>
       <property name="y" type="float" value="66"/>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
  <image source="tiles/tile2.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="3" name="collision" x="15.0398" y="1.81818">
    <polygon points="0,0 14.7784,5.80779 10.8665,15.6364 -13.0398,10.4242"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="3" type="TileComponent">
  <properties>
   <property name="prefered_color" type="color" value="#ffff0307"/>
   <property name="spawn" type="class" propertytype="user_properties_avian::SpawnBundle">
    <properties>
     <property name="collider" type="class" propertytype="user_properties_avian::ColliderComponent">
      <properties>
       <property name="damage_per_second" type="float" value="12"/>
      </properties>
     </property>
    </properties>
   </property>
   <property name="test" type="class" propertytype="avian2d::position::Position">
    <properties>
     <property name="0" type="class" propertytype="glam::Vec2">
      <properties>
       <property name="x" type="float" value="41"/>
       <property name="y" type="float" value="45"/>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
  <image source="tiles/tile3.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="4" type="TileComponent">
  <properties>
   <property name="prefered_color" type="color" value="#ffff0307"/>
   <property name="spawn" type="class" propertytype="user_properties_avian::SpawnBundle">
    <properties>
     <property name="collider" type="class" propertytype="user_properties_avian::ColliderComponent">
      <properties>
       <property name="damage_per_second" type="float" value="12"/>
      </properties>
     </property>
    </properties>
   </property>
   <property name="test" type="class" propertytype="avian2d::position::Position">
    <properties>
     <property name="0" type="class" propertytype="glam::Vec2">
      <properties>
       <property name="x" type="float" value="47"/>
       <property name="y" type="float" value="21"/>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
  <image source="tiles/tile4.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="2" name="collision" x="10.7273" y="20" width="20.7273" height="10.3636">
    <ellipse/>
   </object>
  </objectgroup>
 </tile>
 <tile id="5" type="TileComponent">
  <properties>
   <property name="prefered_color" type="color" value="#ff0fff23"/>
   <property name="spawn" type="class" propertytype="user_properties_avian::SpawnBundle">
    <properties>
     <property name="collider" type="class" propertytype="user_properties_avian::ColliderComponent">
      <properties>
       <property name="damage_per_second" type="float" value="12"/>
      </properties>
     </property>
    </properties>
   </property>
   <property name="test" type="class" propertytype="avian2d::position::Position"/>
  </properties>
  <image source="tiles/tile5.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="0" y="0" width="32" height="32"/>
  </objectgroup>
 </tile>
 <tile id="6" type="TileComponent">
  <properties>
   <property name="prefered_color" type="color" value="#ff0fff23"/>
   <property name="spawn" type="class" propertytype="user_properties_avian::SpawnBundle">
    <properties>
     <property name="collider" type="class" propertytype="user_properties_avian::ColliderComponent">
      <properties>
       <property name="damage_per_second" type="float" value="12"/>
      </properties>
     </property>
    </properties>
   </property>
   <property name="test" type="class" propertytype="avian2d::position::Position"/>
  </properties>
  <image source="tiles/tile6.png" width="32" height="32"/>
  <objectgroup draworder="index" id="2">
   <object id="1" name="collision" x="6.83272" y="1.23615" width="30.1959" height="7.93289" rotation="54.8082"/>
  </objectgroup>
 </tile>
</tileset>
