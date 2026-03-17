melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  b2 c2 b4 a4 g4 a4 f2 e2 r2 \break

  % Line 2
  g2 a2 b4 g4 a4 c4 b2 a2 r2 \break

  % Line 3
  a2 c2 b4 a4 a4 gis4 a2 r2 \break

  % Line 4
  e2 f2 g4 a4 a4 gis4 a2 r2 \break

  % Line 5
  a2 g2 a4 b4 c4 b4 a2 g2 r2 \break

  % Line 6
  a2 g2 e4 f4 g4 e4 f2 e2 r2 \break

  % Line 7
  e2 a2 g4 g4 a4 b4 c2 b2 r2 \break

  % Line 8
  g2 a2 g4 e4 g4 g4 f2 e1 \bar "|."
}
