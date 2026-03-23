melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d4 g4 g4 a4 b2 g4 c4 c4 b4 a2 \break

  % Line 2
  d,4 g4 g4 a4 b2 c4 d4 b4 a4 g2 \break

  % Line 3
  d4 d4 d4 e4 fis4 fis4 g4 a4 a4 b4 c2 \break

  % Line 4
  d,4 g4 g4 a4 b4 b4 c4 d4 b4 a4 g2 \bar "|."
}
