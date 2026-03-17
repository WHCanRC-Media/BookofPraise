melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a2 b4 d4 cis4 b2 a2 gis4 a2 r2 \break

  % Line 2
  d2 cis4 b4 d2 a2 b4 cis4 d2 r2 \break

  % Line 3
  a2 b4 b4 a2 fis2 g4 a4 b2 a2 r2 \break

  % Line 4
  fis2 a4 a4 fis2 d2 g4 fis4 e2 d2 r2 \break

  % Line 5
  a2 b4 cis4 d4 b4 a2 g2 fis2 r2 \break

  % Line 6
  fis2 g4 a4 b2 a4 g4 fis2 e2 r2 \break

  % Line 7
  a2 b4 cis4 a4 d4 cis2 b2 a2 r2 \break

  % Line 8
  fis2 g4 a4 e4 g4 fis2 e2 d1 \bar "|."
}
