melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'4 c4 g4 a4 e4 f4 g4 e2 c4 \break

  % Line 2
  e4 g4 c4 b4 a4 d4 c4 b2. \break

  % Line 3
  g4 c4 g4 a4 e4 f4 g4 e2 c4 \break

  % Line 4
  e4 d4 g4 b4 g4 c4 a4 g2. \break

  % Line 5
  c4 b4 a4 g4 e4 d4 g4 f2 e4 \break

  % Line 6
  d4 e4 f4 g4 e4 a4 a4 g2. \break

  % Line 7
  g4 a4 c4 b4 a4 d4 c4 c2 b4 \break

  % Line 8
  b4 c4 a4 g4 c4 e4 d4 c2. \bar "|."
}
