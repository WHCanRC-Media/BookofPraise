melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 a4 a4 d4 c4 b2 a4 c4 b4 a4 a4 gis4 a2 r2 \break

  % Line 2
  a2 a4 a4 bes2 g2 f4 f4 g4 a4 d,4 f4 e2 d2 r2 \break

  % Line 3
  f2 e4 d4 f4 g4 a2 a4 c4 b4 c4 d4 b4 a2 r2 \break

  % Line 4
  d,2 a'4 a4 g2 f2 e4 a4 a4 g4 f4 d4 e2 d1 \bar "|."
}
