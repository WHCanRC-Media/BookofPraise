melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a'2 g4 e4 f4 f4 g4 a2 gis4 a2 r2 \break

  % Line 2
  d,2 e4 f4 g2 a2 d,2 r2 \break

  % Line 3
  e2 g4 g4 f4 f4 e2 r2 \break

  % Line 4
  g2 a4 b4 c2 b2 a4 g4 a2 b2 r2 \break

  % Line 5
  a2 d4 c4 b4 a4 g2 a2 r2 \break

  % Line 6
  d,2 e4 f4 g4 g4 f2 e1 \bar "|."
}
