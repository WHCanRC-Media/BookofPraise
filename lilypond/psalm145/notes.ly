melody = \relative c'' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a2 b4 g4 a2 d2 c4 a4 b4 b4 a2 r2 \break

  % Line 2
  a2 b4 a4 fis2 a2 g4 fis4 e4 e4 d2 r2 \break

  % Line 3
  d2 a'2 b2 g4 a4 c4 b2 a2 gis4 a2 r2 \break

  % Line 4
  a2 b2 g2 e4 g4 a4 g4 fis2 e2 d2 r2 \break

  % Line 5
  g2 fis4 g4 a2 d2 a4 b4 c4 d4 b2 a2 r2 \break

  % Line 6
  c2 a2 b2 a4 fis4 g4 b4 a4 g4 fis2 e2 r2 \break

  % Line 7
  e2 fis4 g4 a2 b2 a4 g4 a4 b4 c2 b2 r2 \break

  % Line 8
  d2 c4 b4 a2 b2 a4 g4 fis4 d4 e2 d1 \bar "|."
}
