melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 g4 g4 d2 d2 f4 g4 a4 g4 f2 g2 r2 b2 a2 g2 f2 d2 f4 g4 a4 g4 f2 g2 r2 \break

  % Line 2
  d2 d4 d4 g,2 c2 c4 b4 a4 g4 f2 d4 \break

  % Line 3
  g4 g4 f4 g2 f2 g4 f4 g4 a4 b2 a2 r2 \break

  % Line 4
  b2 a4 g4 f2 d2 f4 f4 g4 f4 e2 d2 r2 \break

  % Line 5
  d2 c4 b4 a2 f2 g4 b4 c4 b4 a2 g1 \bar "|."
}
