melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 d4 d4 f4 f4 g4 g4 a2 r2 \break

  % Line 2
  a2 b4 c4 a4 d2 c4 b2 a2 r2 \break

  % Line 3
  a2 g4 f4 e4 f4 g4 a4 g2 f2 r2 \break

  % Line 4
  f2 g4 g4 a2 c2 c4 b4 a2( f2) e2 r2 \break

  % Line 5
  a2 g4 e4 g2 f4 d4 e2 d1 \bar "|."
}
