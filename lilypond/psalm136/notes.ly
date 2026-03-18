melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 d2 d2 g4 a4 b4 c4 d2 r2 \break

  % Line 2
  b2 d2 c4 b4 g2 a2 g2 \break

  % Line 3
  b2 a2 g4 g4 c4 c4 b2 r2 \break

  % Line 4
  g2 a2 fis4 g4 r4 e4 e4 d1 \bar "|."
}
