melody = \relative c'' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 e4 d4 g4 g4 a4 c4 b2 a2 r2 \break

  % Line 2
  d2 b4 b4 g4 c4 b2 a2 g2 r2 \break

  % Line 3
  g2 e4 d4 g4 g4 a4 c4 b2 a2 r2 \break

  % Line 4
  d2 c4 b4 a4 g4 g4 fis4 g2 r2 \break

  % Line 5
  d2 d4 c4 b2 a2 g4 fis4 e2 d2 r2 \break

  % Line 6
  d2 g4 g4 fis4 e4 g2 a2 b2 r2 \break

  % Line 7
  g2 g4 a4 b4 g4 c4 c4 b2 a2 r2 \break

  % Line 8
  d2 c4 b4 a4 g4 g4 fis4 g1 \bar "|."
}
