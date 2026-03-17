melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 f2 e2 d4 d4 e4 f4 g2 r2 \break

  % Line 2
  c2 c4 a4 c2 b2 a4 a4 g2 r2 \break

  % Line 3
  g2 e2 f2 g4 g4 a2 c2 b2 r2 \break

  % Line 4
  g2 c4 b4 c4 d2 c2 b4 c2 r2 \break

  % Line 5
  c2 b2 a2 g4 a4 g4 f4 e2 d2 r2 \break

  % Line 6
  g2 a2 b2 c4 b4 a4 g2 fis4 g1 \bar "|."
}
