# Brickhollow: Open-World-Insel (Plan; Meilenstein 1 fertig)

## Rahmen
- Brickhollow, Route 1 und Stonecrest bleiben **unverändert**. Sie liegen künftig an der Südspitze einer großen Insel.
- **Schlauch zuerst:** Das Nordtor hinter Stonecrest bleibt zu, bis das erste Voxelmon freigeschaltet ist. Ein Wächter oder ein Tor mit Begründung erklärt das. Danach öffnet sich die Insel.
- **Ziel-Hardware:** dein Handy. Budget etwa 40–60 Tsd. Dreiecke pro Bild; die Engine-Taktung regelt dann 30–45 fps.
- **Großer Vorteil:** Unsere Kamera ist fest von oben (etwa 16 m), man sieht nur rund 60 m weit. Rowanmark zeichnet bis 2,5 km. Wir müssen nur einen kleinen Umkreis wirklich bauen.

## 1. Inselform und Gelände („Höhengesetz“)
- Eine einzige, reine Funktion liefert für jede Stelle Höhe, Region, Wald-Dichte und Uferabstand. Sie wird beim Start einmal in ein grobes Gitter gebacken (etwa 4 Noppen pro Feld; Rowanmark: 0,5 s).
- **Küste:** signierter Abstand nach dem Brick-Ages-Muster. Kapseln zwischen den Regionsmittelpunkten, Rauschen dazu, ein paar Buchten. Am Ufer ein Sandstreifen, danach eine flache Schelfstufe ins Meer.
- **Höhen:**
  - Hügel, ein Gebirge mit Pass, ein See und Flüsse; die Flüsse als von Hand gezogene Linien, die ins Gelände eingeschnitten werden.
  - Orte bekommen flache Plattformen mit weichem Übergang.
  - Wege bekommen „Täler“ (Abstandsfeld zur Wegelinie).
- **LEGO-Stufen:** Die Höhe wird in Platten- bzw. Steinschritte gerundet. Kleine Stufen bekommen die Farbe der Fläche darüber, nur große Kanten werden zu Fels mit Schichtstreifen. Optional sitzen an Stufenfüßen Schrägsteine (Brick Ages).
- **Farben:** großflächige Rausch-Flecken statt Zufall pro Noppe, Höhenbänder (Gras → Fels → Schnee) und ±3 % Helligkeit pro Stein, damit Wände als einzelne Steine lesbar sind.

## 2. Darstellung (Leistung)
- **Chunks** von 64×64 Noppen:
  - Oberflächen zu großen Rechtecken zusammengefasst (greedy meshing), Seitenwände als zusammenhängende Streifen.
  - Eine Halb-Auflösungs-Version für weiter weg; ohne Noppen und ohne Teile unter etwa 1 Noppe.
  - Die gröbere Stufe bleibt stehen, bis die feine fertig ist, damit es nie Löcher gibt.
- **Noppen:** echte Geometrie nur im Nahbereich (etwa 12–16 Noppen), weiter weg per Normalmap bzw. Shader, wie unsere Grundplatte heute schon.
- **Bauen im Lauf:** nur das grobe Gitter beim Start. Chunks entstehen beim Herumlaufen mit festem Millisekunden-Budget pro Bild. Deko wird pro Chunk erst beim ersten Sichtbarwerden verteilt, mit fester Obergrenze pro Chunk.
- **Statisches pro Chunk zusammenfassen** (Gelände, Häuser, Deko). Belebtes (Figuren, Voxelmon, Gras) nur im Nahbereich.
- **Schatten** nur nah und selten aktualisiert. Leben (Figuren, Tiere) wird nur in etwa 60 m Umkreis simuliert.
- **Engine-Wünsche** (an die Engine-Session):
  - Chunk-Streaming mit Zeitbudget, bzw. bestätigen, was terrain.rs und streaming.rs schon können.
  - Gemeinsame Noppen-Darstellung für viele Chunks.
  - Eventuell eine Brick-Chunk-Meshing-Hilfe in der Engine.
  - Vorwärmen für neue Materialien.

## 3. Orte, Wege, Häuser
- **Etwa 6–8 Regionen** mit eigenem Charakter, z. B.:
  - Wald,
  - Seenland,
  - Gebirge/Schnee,
  - Canyon/Wüste,
  - Piratenbucht,
  - Sumpf,
  - Burgruine,
  - Vulkan.

  Jede Region hat:
  - eine kleine Siedlung,
  - einen Voxelmon-Lebensraum (eigene Arten),
  - einen Kartenspiel-Gegner bzw. Champion,
  - einen Wegstein.
- **Orte** als feste Rechtecke mit Toren (Rowanmark-Prinzip: kontrollierbar statt Zufall), Wege als gezogene Linien von Tor zu Tor.
  - Selbsttests: keine Überlappungen; Tore nah am Weg; alles über dem Meeresspiegel.
  - Ein Nachbearbeitungsschritt glättet die Wege auf begehbare Steigung.
  - Brücken entstehen automatisch, wo ein Weg Wasser kreuzt.
- **Häuser prozedural** aus etwa 6–10 Parametern (Brick-Ages-Prinzip):
  - Wände im Läuferverband mit Fenster- und Türlöchern;
  - Fenster mit Rahmen, Läden und Blumenkasten nach Regionsstil;
  - Dach als Stufengiebel, Walmdach oder Gaube;
  - Grundstück mit Zaun, Weg und Beet;
  - Farben aus kuratierten Dreiergruppen.

  Praktisch kein Speicher pro Haus.
- **Gegen Z-Fighting** (dort ist Brick Ages unsauber): ein Belegungsgitter pro Chunk. Nie zwei Steine in dieselbe Zelle; ein späterer Stein ersetzt den früheren. Deko-Flächen in einem eigenen Durchgang. Und unser automatischer `zfight`-Test läuft pro Region mit.

## 4. Spielablauf auf der Insel
- **Entdecken:** Orte und Regionen werden beim Betreten entdeckt, mit Titelkarte und Belohnung.
- **Wegsteine** als Schnellreise; sie setzen auch den Wiedereinstiegspunkt.
- **Begegnungs-Spielleiter:** Alle paar Minuten Wegstrecke gibt es voraus etwas: wilde Voxelmon, ein Trainer, ein Händler, eine Truhe. Nie direkt an Orten.
- **Fortschritt:** Regionen locker gestaffelt über Voxelmon-Stärke bzw. Karten. Die Hauptlinie führt von Champion zu Champion; dazu Nebenaufgaben und Sammelsachen (goldene Noppen gibt es schon).
- **Tag und Nacht, Regen:** bleiben wie im Dorf.
- **Speichern:** Position, entdeckte Orte, Wegsteine und Aufgabenstand im bestehenden Speicherstand.

## 5. Karte
- **Weltkarte** einmal aus dem Gitter gemalt, im Stil einer Bauanleitung bzw. Pergamentkarte:
  - Regionsfarben mit Schattierung, Küstenlinie, Wege;
  - Symbole (Bäume, Berge, Häuser), sortiert, damit sie sich bildhaft überlappen;
  - Raster A–H / 1–8.
- **Minikarte** als runder Ausschnitt desselben Bildes.
- **Unentdecktes** erscheint als „?“. Markierungen für Aufgaben, Wegsteine, Champions und die eigene Position.
- **Bedienung:** am Handy per Knopf, am PC mit M. Antippen eines Wegsteins reist dorthin.

## 6. Meilensteine
1. **Begehbare Insel:**
   - Tor hinter Stonecrest mit Bedingung „erstes Voxelmon“;
   - Höhengesetz, Küste und Meer;
   - Brick-Chunks mit LOD und Streaming;
   - Weltkarte v1.
   - Auf dem Handy messen (`?measure`).
2. **Orte und Wege:** Regionen, Siedlungen mit prozeduralen Häusern, Wege, Brücken, Vegetation, Z-Fighting-Tests.
3. **Spiel:** Voxelmon-Lebensräume, Champions, Begegnungs-Spielleiter, Wegsteine und Schnellreise, Entdecken, Aufgaben.
4. **Feinschliff:** Wasser und Strände, Atmosphäre pro Region, Leistungsrunde auf dem Handy.

Jeder Meilenstein: Tests, Bildschirmfotos, Messung auf dem Handy, Commit.

**Stand Meilenstein 1 (21.11.2026, fertig):** Nordtor hinter Stonecrest (Wache, Bedingung „erstes Voxelmon“, Tür-Animation, `island {gate}`), Höhengesetz `island.rs` (Gitter in ~50 ms gebacken, Selbsttests), drei flache Plätze (Brickton, Port Saltwick, Dewmere) mit gestufter Hauptstraße, Brick-Chunks mit grober/feiner Stufe, Noppen nah, Kollision, Meer mit Watzone (`islandview.rs`), Weltkarte v1 (`worldmap.rs`, M / Menü). Tests `tests/island.txt`, `tests/island_touch.txt`. Gemessen (Desktop, Debug-Build): Backen ~50 ms, Chunk fertigstellen ≤ 3,3 ms, Bau pro Bild ≤ 4,7 ms, 40–50 Tsd. Dreiecke (Handy-Ansicht 19–31 Tsd.). Noch offen: Messung auf dem echten Handy (`?measure`), Seen/Flüsse, Vegetation, Brücken (Meilenstein 2).

## Entscheidungen (21.11.2026)
- **Tor:** Bedingung ist „erstes Voxelmon freigeschaltet“. Das braucht viel EXP, egal aus welchen Kämpfen (Dex, Lumi oder Pip).
- **Größe:** Wir starten mit etwa **500 × 500 m**, das sind rund 1.300 × 1.300 Noppen. Lieber dicht und lebendig wie Gothic 1 statt riesig und leer wie Gothic 3 oder Just Cause 2. Weitere Inseln oder Dungeons kommen später, nicht im ersten Schritt.
- **Gelände:** LEGO-Stufen aus Platten und Steinen, dazu einzelne sanftere Hügel.
- **Orte wie bei Pokémon Rot/Blau:** von Hand gestaltet und mit eigenem Charakter, nicht zufällig:
  - eine **Großstadt** mit großem Einkaufszentrum und Wolkenkratzern;
  - eine **Hafenstadt am Meer** mit Boot (später der Weg zur nächsten Insel);
  - ein **verträumtes, stilles Dorf**;
  - der **Norden** in einer anderen Klimazone (kühl, Nadelwald, Schnee auf den Höhen), der **Süden** warm (Strand, Palmen).
- **Lebendige Welt** statt Masse: Bewohner mit Tagesablauf und Reaktionen auf den Spieler (wie das Dorfleben in life.rs); Aufgaben, die man in der Welt findet. Die prozeduralen Teile (Häuser-Regeln, Vegetation, Geländedetails) dienen als Füllung, nicht als Inhalt.
