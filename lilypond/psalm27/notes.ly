melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 fis2 g2 a4 a4 d4 d4 c4 d4 b2 a2 r2 \break

  % Line 2
  e2 fis2 a2 g4 e4 a4 g4 fis2 e2 d2 r2 \break

  % Line 3
  a'2 fis2 g2 a4 a4 d4 d4 c4 d4 b2 a2 r2 \break

  % Line 4
  e2 fis2 a2 g4 e4 a4 g4 fis2 e2 d2 r2 \break

  % Line 5
  d'2 c4 b4 a4 a4 c4 c4 b2 c2 d2 \break

  % Line 6
  g,2 a4 b4 c4 c4 b4 a4 a4 gis4 a2 r2 \break

  % Line 7
  a2 c4 c4 b4 g4 a4 e4 g2 fis2 e2 r2 \break

  % Line 8
  c'2 b2 a2 b4 b4 a4 g4 fis2 e2 d1 \bar "|."
}
