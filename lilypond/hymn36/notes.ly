melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'4 g4. a8 b4 a4 g4 c4 a4 \break

  % Line 2
  a4 b4 d4 d4 cis4 d2. \break

  % Line 3
  g,4 c4 c4 b4 a4 g4 a4 fis4 \break

  % Line 4
  b4 e,8( fis8) g4 g4 fis4 g2. \bar "|."
}
