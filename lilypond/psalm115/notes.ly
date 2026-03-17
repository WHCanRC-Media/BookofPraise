melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a2 c4 b4 a2 d,2 f4 f4 g4 g4 a2 \break

  % Line 2
  r4 a4 c2 c2 d4 d4 c4 a4 bes2 g2 f2 r2 r4 \break

  % Line 3
  a2 g4 f4 e4 f4 d2 c2 r2 r4 \break

  % Line 4
  f2 f4 g4 a2 b2 c4 d4 c4 b4 a2 r2 \break

  % Line 5
  c2 c4 b4 a2 e2 f4 a4 g4 f4 e2 r2 \break

  % Line 6
  a2 g4 e4 f4 g4 e2 d1 \bar "|."
}
