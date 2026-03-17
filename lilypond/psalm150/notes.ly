melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 fis2 e4 d4 fis4 g4 a2 \break

  % Line 2
  d2 fis2 e4 d4 fis4 g4 a2 r2 \break

  % Line 3
  fis2 b2 a4 d4 d4 cis4 d2 \break

  % Line 4
  a2 d2 b4 a4 a4 gis4 a2 r2 \break

  % Line 5
  fis2 a2 b4 a4 g4 fis4 e2 d2 r2 \break

  % Line 6
  d2 g2 fis4 e4 fis4 g4 a2 r2 \break

  % Line 7
  d2 g2 fis4 e4 fis4 g4 a2 r2 \break

  % Line 8
  d2 b2 a4 g4 fis4 d4 e2 d1 \bar "|."
}
