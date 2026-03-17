melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 e2 f2 g4 g4 a4 c4 c4 b4 c2 r2 \break

  % Line 2
  c2 a2 c2 b4 g4 a4 b4 c2 a2 g2 r2 \break

  % Line 3
  e2 f4 g4 a2 g2 f4 d4 e4 e4 d2 r2 \break

  % Line 4
  g2 g4 g4 a2 c2 c4 b4 c2 a2 g1 \bar "|."
}
