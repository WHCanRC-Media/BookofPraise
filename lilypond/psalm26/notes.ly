melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e2 a2 g2 f4 f4 e2 r2 \break

  % Line 2
  a2 g4 e4 f4 g4 a2 r2 \break

  % Line 3
  c2 b4 a4 g2 e2 f4 d4 e2 r2 \break

  % Line 4
  g2 a4 b4 c4 b4 a2 g2 r2 \break

  % Line 5
  c2 b4 d4 a4 c4 b2 a2 r2 \break

  % Line 6
  a2 c2 b2 a4 g4 f4 f4 e1 \bar "|."
}
