melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 fis4 g4 a2 d2 cis4 d4 b2 a2 r2 \break

  % Line 2
  a2 d4 cis4 b2 a2 fis4 d4 e2 d2 r2 \break

  % Line 3
  g2 fis4 g4 a4 b2 a2 gis4 a2 r2 \break

  % Line 4
  a2 fis4 g4 a4 b2 a2 gis4 a2 r2 \break

  % Line 5
  a2 d4 cis4 b4 a4 g2 fis2 e2 r2 \break

  % Line 6
  a2 fis4 b4 a4 g4 fis2 e2 d1 \bar "|."
}
