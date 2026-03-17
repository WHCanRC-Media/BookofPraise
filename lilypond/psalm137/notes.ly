melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 f4 g4 a2 a2 a4 g4 a4 b4 c2 a2 r2 \break

  % Line 2
  c2 a4 g4 f2 g2 g4 a4 g4 f4 e2 d2 \break

  % Line 3
  r4 d4 f2 a2 g2 f2 e4 d4 e4 e4 d2 r2 \break

  % Line 4
  a2 a4 g4 f2 a2 c4 c4 b4 b4 a2 r2 \break

  % Line 5
  f2 e4 d4 a'2 c2 b4 a4 g4 a4 f2 e2 r2 \break

  % Line 6
  d2 f4 g4 a2 g2 g4 a4 g4 f4 e2 d1 \bar "|."
}
