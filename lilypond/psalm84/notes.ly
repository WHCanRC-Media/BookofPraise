melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 a'2 g4 fis4 e4 d4 e2 fis2 r2 \break

  % Line 2
  fis2 b4 b4 a4 fis4 a2 g2 fis2 r2 \break

  % Line 3
  a2 g4 fis4 e4 a4 g4 fis4 e2 d2 r2 \break

  % Line 4
  d2 fis4 g4 a2 b2 a4 gis4 a2 r2 \break

  % Line 5
  d,2 fis4 g4 a2 b2 a4 gis4 a2 r2 \break

  % Line 6
  d2 cis4 b4 a2 fis2 g4 a4 b2 a2 r2 \break

  % Line 7
  a2 a4 g4 fis2 b2 a4 g4 fis2 r2 \break

  % Line 8
  d2 fis4 a4 e4 g4 fis2 e2 d1 \bar "|."
}
