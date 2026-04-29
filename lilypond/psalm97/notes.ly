melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 b4 d4 d4 cis4 d2 r2 \break

  % Line 2
  d2 b4 cis4 d4 b4 a2 \break

  % Line 3
  a2 d4 b4 cis4 d4 b2 a2 r2 \break

  % Line 4
  d,2 fis4 a4 g4 fis4 e2 d2 r2 \break

  % Line 5
  fis2 e4 a4 a4 gis4 a2 r2 \break

  % Line 6
  fis2 e4 a4 a4 gis4 a2 r2 \break

  % Line 7
  fis2 b4 b4 a2 g2 fis4 \break

  % Line 8
  fis4 b4 b4 a2 g2 fis2 r2 \break

  % Line 9
  a2 d,4 g4 fis2 e2 d1 \bar "|."
}
