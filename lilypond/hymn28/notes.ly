melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r4 g'4 g4 b4 a4 g4 a4 a4 b2 \break

  % Line 2
  g2 b4 c4 d4 b4 a2 g2 \break

  % Line 3
  r4 g4 g4 b4 a4 g4 a4 a4 b2 \break

  % Line 4
  g2 b4 c4 d4 b4 a2 g2 \break

  % Line 5
  r4 b4 b4 a4 g4 fis4 g4 e4 d2 \break

  % Line 6
  r4 d4 g4 g4 g4 fis4 g4 a4 b2 \break

  % Line 7
  g2 b4 c4 d4 b4 a2 g1 \bar "|."
}
