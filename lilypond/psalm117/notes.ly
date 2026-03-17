melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a2 d4 d4 cis4 a4 b4 cis4 d2 r2 \break

  % Line 2
  d2 cis4 b4 a4 g4 fis2 e2 d2 r2 \break

  % Line 3
  d2 e4 fis4 g4 a4 a4 gis4 a2 r2 \break

  % Line 4
  a2 b4 b4 d4 d4 cis2 b2 r4 a2 r2 \break

  % Line 5
  a2 g4 fis4 b4 a4 g2 fis2 e2 r2 \break

  % Line 6
  e2 fis4 g4 a4 b2 a2 gis4 a1 \bar "|."
}
