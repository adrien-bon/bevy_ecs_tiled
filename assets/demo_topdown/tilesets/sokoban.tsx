<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="sokoban" tilewidth="64" tileheight="64" tilecount="104" columns="13">
 <image source="../kenney_sokoban-pack/Tilesheet/sokoban_tilesheet.png" width="832" height="512"/>
 <tile id="1">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
   <property name="Destructible" type="class" propertytype="demo_topdown::interaction::Destructible"/>
  </properties>
 </tile>
 <tile id="2">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
   <property name="Gate" type="class" propertytype="demo_topdown::interaction::Gate">
    <properties>
     <property name="channel" type="class" propertytype="demo_topdown::interaction::Channel">
      <properties>
       <property name=":variant" propertytype="demo_topdown::interaction::Channel:::Variant" value="Red"/>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="3">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
   <property name="Gate" type="class" propertytype="demo_topdown::interaction::Gate">
    <properties>
     <property name="channel" type="class" propertytype="demo_topdown::interaction::Channel">
      <properties>
       <property name=":variant" propertytype="demo_topdown::interaction::Channel:::Variant" value="Blue"/>
      </properties>
     </property>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="38">
  <properties>
   <property name="Button" type="class" propertytype="demo_topdown::interaction::Button">
    <properties>
     <property name="channel" type="class" propertytype="demo_topdown::interaction::Channel">
      <properties>
       <property name=":variant" propertytype="demo_topdown::interaction::Channel:::Variant" value="Red"/>
      </properties>
     </property>
     <property name="pressed_tile" type="int" value="37"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="51">
  <properties>
   <property name="Button" type="class" propertytype="demo_topdown::interaction::Button">
    <properties>
     <property name="channel" type="class" propertytype="demo_topdown::interaction::Channel">
      <properties>
       <property name=":variant" propertytype="demo_topdown::interaction::Channel:::Variant" value="Blue"/>
      </properties>
     </property>
     <property name="pressed_tile" type="int" value="50"/>
    </properties>
   </property>
  </properties>
 </tile>
 <tile id="84">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
 <tile id="85">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
 <tile id="86">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
 <tile id="87">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
 <tile id="97">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
 <tile id="98">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
 <tile id="99">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
 <tile id="100">
  <properties>
   <property name="Obstacle" type="class" propertytype="demo_topdown::navigation::Obstacle"/>
  </properties>
 </tile>
</tileset>
