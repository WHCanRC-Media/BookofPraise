melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  fis2 fis4 fis4 a2 g4( fis4) e4( fis4) g2 fis2 \break

  % Line 2
  fis2 b2. b4 a2 gis2 a1 \break

  % Line 3
  fis2 fis4 fis4 a2 g4( fis4) e4( fis4) g2 fis2 \break

  % Line 4
  fis2 b2. b4 a4( fis4) e2 fis1 \bar "|."
}
